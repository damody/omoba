# Acceptance failed

scripts/owned_acceptance.lua:56: acceptance child exited with Windows code 1
stack traceback:
	[C]: in function 'assert'
	scripts/owned_acceptance.lua:56: in function <scripts/owned_acceptance.lua:44>
	[C]: in function 'xpcall'
	scripts/owned_acceptance.lua:44: in function 'owned_acceptance.run'
	scripts/run_owned_acceptance.lua:7: in main chunk
	[C]: in ?

Only recorded original lifetimes were retired. Intermediate log text and final summary cannot override a failed Windows exit.
