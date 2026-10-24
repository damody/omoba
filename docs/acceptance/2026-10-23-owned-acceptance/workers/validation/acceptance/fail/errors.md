# Acceptance failed

scripts/tests/../owned_acceptance.lua:56: acceptance child exited with Windows code 1
stack traceback:
	[C]: in function 'assert'
	scripts/tests/../owned_acceptance.lua:56: in function <scripts/tests/../owned_acceptance.lua:44>
	[C]: in function 'xpcall'
	scripts/tests/../owned_acceptance.lua:44: in function 'owned_acceptance.run'
	scripts/tests/owned_acceptance_test.lua:13: in function <scripts/tests/owned_acceptance_test.lua:9>
	[C]: in function 'xpcall'
	scripts/tests/owned_acceptance_test.lua:9: in main chunk
	[C]: in ?

Only recorded original lifetimes were retired. Intermediate log text and final summary cannot override a failed Windows exit.
