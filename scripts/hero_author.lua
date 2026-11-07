-- Hero authoring tool (plan E2): three deterministic subcommands.
--
--   tools\lua\lua.exe scripts\hero_author.lua index    --source <checkout> [...]   (hero_source_index index)
--   tools\lua\lua.exe scripts\hero_author.lua validate <package_dir> [--catalog f] [--requirements f]
--   tools\lua\lua.exe scripts\hero_author.lua build    <package_dir> [--slice id,id] [--execute]
--   tools\lua\lua.exe scripts\hero_author.lua shadow   <package_dir> --tree <dir> [--slice id,id]
--
-- shadow: copies the crates base_content needs into a private tree and builds
--   the package there (content + gen_hero_registry only, no UE build, nothing
--   staged), so handlers and parity soaks can be tested with a private
--   CARGO_TARGET_DIR without touching the shared checkout.
-- validate: package schema v2, provenance against the source catalog, capability
--   statuses against the D3 registry, missing Rust handlers, and a generation
--   dry run through scripts/gen_hero_registry.lua on a scratch copy of the templates.
-- build: same checks, then (only with --execute) writes templates/authored_heroes.lua
--   and the include hooks, runs gen_hero_registry, build_ue_moba, the staged-copy
--   verification and the compiled-content contract. Any failure restores the
--   content files and the built/staged DLLs, so the previous build stays usable.
--   Without --execute it prints the plan and changes nothing.
local script = debug.getinfo(1, "S").source:sub(2)
local dir = script:match("^(.*)[/\\]") or "."
package.path = dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local platform = b.lib("platform")
local cli = require("hero_author.cli")
os.exit(cli.run(arg, cli.default_deps((b.root:gsub("\\", "/")), platform.lua_executable)))
