var sourcesIndex = JSON.parse('{\
"fe":["",[["task",[],["build.rs","check.rs","mod.rs","new.rs","utils.rs"]]],["main.rs"]],\
"fe_abi":["",[],["contract.rs","event.rs","function.rs","lib.rs","types.rs"]],\
"fe_analyzer":["",[["db",[["queries",[],["contracts.rs","enums.rs","functions.rs","impls.rs","ingots.rs","module.rs","structs.rs","traits.rs","types.rs"]]],["queries.rs"]],["namespace",[],["items.rs","mod.rs","scopes.rs","types.rs"]],["traversal",[],["assignments.rs","borrowck.rs","call_args.rs","const_expr.rs","declarations.rs","expressions.rs","functions.rs","matching_anomaly.rs","mod.rs","pattern_analysis.rs","pragma.rs","types.rs","utils.rs"]]],["builtins.rs","constants.rs","context.rs","db.rs","display.rs","errors.rs","lib.rs","operations.rs"]],\
"fe_codegen":["",[["db",[["queries",[],["abi.rs","constant.rs","contract.rs","function.rs","types.rs"]]],["queries.rs"]],["yul",[["isel",[],["context.rs","contract.rs","function.rs","inst_order.rs","mod.rs","test.rs"]],["legalize",[],["body.rs","critical_edge.rs","mod.rs","signature.rs"]],["runtime",[],["abi.rs","contract.rs","data.rs","emit.rs","mod.rs","revert.rs","safe_math.rs"]]],["mod.rs","slot_size.rs"]]],["db.rs","lib.rs"]],\
"fe_common":["",[["utils",[],["humanize.rs","keccak.rs","mod.rs","ron.rs"]]],["db.rs","diagnostics.rs","files.rs","lib.rs","numeric.rs","panic.rs","span.rs"]],\
"fe_compiler_test_utils":["",[],["lib.rs"]],\
"fe_compiler_tests":["",[],["lib.rs"]],\
"fe_compiler_tests_legacy":["",[],["lib.rs"]],\
"fe_driver":["",[],["lib.rs"]],\
"fe_library":["",[],["lib.rs"]],\
"fe_mir":["",[["analysis",[],["cfg.rs","domtree.rs","loop_tree.rs","mod.rs","post_domtree.rs"]],["db",[["queries",[],["constant.rs","contract.rs","enums.rs","function.rs","module.rs","structs.rs","types.rs"]]],["queries.rs"]],["graphviz",[],["block.rs","function.rs","mod.rs","module.rs"]],["ir",[],["basic_block.rs","body_builder.rs","body_cursor.rs","body_order.rs","constant.rs","function.rs","inst.rs","mod.rs","types.rs","value.rs"]],["lower",[["pattern_match",[],["decision_tree.rs","mod.rs","tree_vis.rs"]]],["function.rs","mod.rs","types.rs"]],["pretty_print",[],["inst.rs","mod.rs","types.rs","value.rs"]]],["db.rs","lib.rs"]],\
"fe_parser":["",[["grammar",[],["contracts.rs","expressions.rs","functions.rs","module.rs","types.rs"]],["lexer",[],["token.rs"]]],["ast.rs","grammar.rs","lexer.rs","lib.rs","node.rs","parser.rs"]],\
"fe_test_files":["",[],["lib.rs"]],\
"fe_test_runner":["",[],["lib.rs"]],\
"fe_yulc":["",[],["lib.rs"]]\
}');
createSourceSidebar();
