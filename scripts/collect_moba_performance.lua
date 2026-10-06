-- Read formal server, client-runtime, and Unreal logs into one performance summary.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local report = require('moba_performance_report')
local code = report.main(arg)
os.exit(code)
