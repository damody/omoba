local script=debug.getinfo(1,'S').source:sub(2);local dir=script:match('^(.*)[/\\]');package.path=dir..'/?.lua;'..package.path
local b=require('_bootstrap');local args=b.lib('args').parse(arg);local json=b.lib('json');local udp=b.lib('udp');local evidence=b.lib('evidence')
local action=args.action or args.positional[1];assert(action=='profile'or action=='shutdown','--action must be profile or shutdown');local payload={version=1,action=action}
if action=='profile'then payload.team_id=b.lib('args').integer(args['team-id']or args.team_id,'team-id',1,2);payload.profile=assert(args.profile,'--profile required');payload.authoritative_tick=b.lib('args').integer(args['authoritative-tick']or args.authoritative_tick or 0,'authoritative-tick',0);local weights=args['weights-json']or args.weights_json;if weights then payload.weights=evidence.weights20(json.read(weights).weights)end end
udp.send(args['control-addr']or args.control_addr or'127.0.0.1:63200',json.encode(payload))
