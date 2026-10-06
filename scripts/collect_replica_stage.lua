local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
require('_bootstrap')
os.exit(require('moba_replica_stage_report').main(arg))
