def "main pull" [] {
	let backends = open backends.txt | lines; 
	mut res = [];
	for backend in $backends {
		print $"Extracting ($backend)"
		let config = http get $"https://qlever.cs.uni-freiburg.de/api/config/($backend)";
		$config  | save -f $"($backend)-config.yml"
		$res = $config.config.backend | append $res;
		let prefixes = http get $"https://qlever.cs.uni-freiburg.de/api/prefixes/($backend)";
		$prefixes | save -f $"($backend)-prefixes.yml"
	}
	$res | save -f all.yaml;
}

def "main transform" [] {
	let data = (open "all.yaml" | select name slug baseUrl suggestedPrefixes subjectName predicateName objectName suggestSubjectsContextInsensitive suggestPredicatesContextInsensitive suggestObjectsContextInsensitive suggestSubjects suggestPredicates suggestObjects  warmupQuery1 warmupQuery2 warmupQuery3 warmupQuery4 warmupQuery5  entityScorePattern entityNameAndAliasPattern entityNameAndAliasPatternDefault predicateNameAndAliasPatternWithoutContext predicateNameAndAliasPatternWithoutContextDefault predicateNameAndAliasPatternWithContext predicateNameAndAliasPatternWithContextDefault | rename name slug url     prefixMap          hoverName   hoverPredicate hoverObject subjectCompletionQuery          predicateCompletionQuery objectCompletionQuery subjectCompletionQueryContextSensitive  predicateCompletionQueryContextSensitive objectCompletionQueryContextSensitive)
	| upsert prefixMap {|row|
		$row.prefixMap
			| lines
			| each {| line| $line
				| split row -r '\s+'
				| get 1 2
				| upsert 1 {|url| $url | str replace -r '<(.+)>' '${1}'}
			        | upsert 0 {|url| $url | str replace -r '(.*):' '${1}'}
				| {  $in.0: $in.1 }
			} | reduce {|it| merge $it}
	}
	| upsert backend {|row|
		{
			name: $row.name,
			slug: $row.slug,
			url: $row.url,
			healthCheckUrl: ($row.url + "/ping")
		}
	}
	| upsert subjectCompletionQuery {|backend| replace $backend subjectCompletionQuery}
	| upsert predicateCompletionQuery {|backend| replace $backend predicateCompletionQuery}
	| upsert objectCompletionQuery {|backend| replace $backend objectCompletionQuery}
	| upsert objectCompletionQuery {|backend| replace $backend objectCompletionQuery}
	| upsert subjectCompletionQueryContextSensitive {|backend| replace $backend subjectCompletionQueryContextSensitive}
	| upsert predicateCompletionQueryContextSensitive {|backend| replace $backend predicateCompletionQueryContextSensitive}
	| upsert objectCompletionQueryContextSensitive {|backend| replace $backend objectCompletionQueryContextSensitive}
	| upsert queries {|backend| 
		{
			subjectCompletion: $backend.subjectCompletionquery,
			predicateCompletion: $backend.predicateCompletionQuery,
			objectCompletion: $backend.objectCompletionQuery,
			subjectCompletionContextSensitive: $backend.subjectCompletionQueryContextSensitive,
			predicateCompletionContextSensitive: $backend.predicateCompletionQueryContextSensitive,
			objectCompletionContextSensitive: $backend.objectCompletionQueryContextSensitive
		}
	}
	| upsert default false
	# | reject name url
	| select backend prefixMap default queries;



	$data | save -f "transformed.yaml";
		# print ($backend 
		# 	| upsert prefixes {|row| $row.prefixes | lines | each {|x| $x 
		# 		| split chars 
		# 		| split list " " 
		# 		| get 2 3 
		# 		| each {|l| $l | str join} } }
		# 	| select prefixes  | table --expand
		#
		# )
	
}

def replace [backend, query: string] {
	$backend | get $query
		| str replace --all "%WARMUP_QUERY_1%" $backend.warmupQuery1
		| str replace --all "%WARMUP_QUERY_2%" $backend.warmupQuery2
		| str replace --all "%WARMUP_QUERY_3%" $backend.warmupQuery3
		| str replace --all "%WARMUP_QUERY_4%" $backend.warmupQuery4
		| str replace --all "%WARMUP_QUERY_5%" $backend.warmupQuery5
		| str replace --all "%ENTITY_SCORE_PATTERN%" $backend.entityScorePattern
		| str replace --all "%ENTITY_NAME_AND_ALIAS_PATTERN%" $backend.entityNameAndAliasPattern
		| str replace --all "%ENTITY_NAME_AND_ALIAS_PATTERN_DEFAULT%" $backend.entityNameAndAliasPatternDefault
		| str replace --all "%PREDICATE_NAME_AND_ALIAS_PATTERN_WITH_CONTEXT%" $backend.predicateNameAndAliasPatternWithContext
		| str replace --all "%PREDICATE_NAME_AND_ALIAS_PATTERN_WITH_CONTEXT_DEFAULT%" $backend.predicateNameAndAliasPatternWithContextDefault
		| str replace --all "%PREDICATE_NAME_AND_ALIAS_PATTERN_WITHOUT_CONTEXT%" $backend.predicateNameAndAliasPatternWithContext
		| str replace --all "%PREDICATE_NAME_AND_ALIAS_PATTERN_WITHOUT_CONTEXT_DEFAULT%" $backend.predicateNameAndAliasPatternWithContextDefault
		| str replace --all "?qui_entity" "?qlue_ls_entity"
		| str replace --all "?qleverui_entity" "?qlue_ls_entity"
		| str replace --all "?qui_count" "?qlue_ls_count"
		| str replace --all "?qleverui_count" "?qlue_ls_count"
		| str replace --all "?qui_alias" "?qlue_ls_alias"
		| str replace --all "?qleverui_alias" "?qlue_ls_alias"
		| str replace --all "?qui_name" "?qlue_ls_label"
		| str replace --all "?qleverui_name" "?qlue_ls_label"
		| str replace --all "%CURRENT_WORD%" "{{ search_term }}"
		| str replace --all "%CURRENT_SUBJECT%" "{{ subject }}"
		| str replace --all "%CONNECTED_TRIPLES%" "{{ context }}"
		| str replace --all "# IF CURRENT_WORD_EMPTY #" "{% if not search_term %}"
		| str replace --all "# IF CURRENT_SUBJECT_VARIABLE #" "{% if subject is variable %}"
		| str replace --all "# IF !CURRENT_SUBJECT_VARIABLE #" "{% if subject is not variable %}"
		| str replace --all "# IF !CURRENT_WORD_EMPTY #" "{% if search_term %}"
		| str replace --all "# IF CURRENT_SUBJECT_VARIABLE AND CONNECTED_TRIPLES_EMPTY #" "{% if subject is variable and context %}"
		| str replace --all "# IF CURRENT_SUBJECT_VARIABLE AND !CONNECTED_TRIPLES_EMPTY #" "{% if subject is variable and context %}"
		| str replace --all "# IF !CONNECTED_TRIPLES_EMPTY AND CURRENT_SUBJECT_VARIABLE #" "{% if context and subject is variable %}"
		| str replace --all "# IF CONNECTED_TRIPLES_EMPTY AND CURRENT_SUBJECT_VARIABLE #" "{% if not context and subject is variable %}"
		| str replace --all "# ELSE #" "{% else %}"
		| str replace --all "# ENDIF #" "{% endif %}"
		| str replace --all "%PREFIXES%" "{% for prefix in prefixes %}\nPREFIX {{prefix.0}}: <{{prefix.1}}>\n{% endfor %}"

}

def main [] {
	print "run pull to download"
	print "run build to build"
}
