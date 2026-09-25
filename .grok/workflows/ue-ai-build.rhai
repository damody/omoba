let meta = #{
    name: "ue-ai-build",
    description: "Autonomous Unreal feature build via Ultimate Engine Copilot MCP",
    when_to_use: "Unreal Editor is open and you want Grok to implement a game feature without waiting for you",
    phases: [
        #{ title: "Inspect", detail: "read the Unreal project and UECP MCP state" },
        #{ title: "Plan", detail: "turn the goal into concrete Unreal work" },
        #{ title: "Implement", detail: "execute the plan in the live editor" },
        #{ title: "Verify", detail: "confirm the result exists" },
    ],
};

let plan_schema = #{
    "type": "object",
    "required": ["summary", "tasks"],
    "properties": #{
        "summary": #{ "type": "string" },
        "tasks": #{
            "type": "array",
            "maxItems": 8,
            "items": #{
                "type": "object",
                "required": ["id", "title", "done_when"],
                "properties": #{
                    "id": #{ "type": "string" },
                    "title": #{ "type": "string" },
                    "done_when": #{ "type": "string" },
                },
            },
        },
    },
};

let inspect_schema = #{
    "type": "object",
    "required": ["mcp_ready", "notes"],
    "properties": #{
        "mcp_ready": #{ "type": "boolean" },
        "notes": #{ "type": "string" },
    },
};

let verify_schema = #{
    "type": "object",
    "required": ["passed", "evidence", "gaps"],
    "properties": #{
        "passed": #{ "type": "boolean" },
        "evidence": #{ "type": "string" },
        "gaps": #{ "type": "string" },
    },
};

let goal = if args == () { () } else { args.goal };
if goal == () { pause("verification", "Pass args.goal — the Unreal feature Grok should build."); }

let goal_json = json_encode(goal);

phase("Inspect");
let inspect_prompt = "";
inspect_prompt += "Inspect whether Ultimate Engine Copilot (UECP) MCP is usable for Unreal work. ";
inspect_prompt += "The Unreal project is D:/code/omoba/omfue/om.uproject. ";
inspect_prompt += "Use search_tool/use_tool and any MCP server named uecp. ";
inspect_prompt += "Also check http://127.0.0.1:30000/mcp/health through run_terminal_command if needed. ";
inspect_prompt += "Read C:/Users/Damody/AppData/Local/UECP/instances if present. ";
inspect_prompt += "Do not invent MCP status. mcp_ready=true only if you actually reached UECP tools or /mcp/health. ";
inspect_prompt += "Goal (JSON): " + goal_json;
let inspect = agent(inspect_prompt, #{
    label: "inspect-uecp",
    capability_mode: "all",
    output_schema: inspect_schema,
});
let mcp_ready = false;
let inspect_notes = "";
if inspect != () && inspect.success && inspect.output != () {
    if inspect.output.mcp_ready == true { mcp_ready = true; }
    if inspect.output.notes != () { inspect_notes = inspect.output.notes; }
}
log("mcp_ready=" + json_encode(mcp_ready) + " notes=" + inspect_notes);

phase("Plan");
let plan_prompt = "";
plan_prompt += "You are the Unreal architect for omfue (om.uproject). ";
plan_prompt += "Turn this goal into at most 8 concrete editor tasks Grok can execute with UECP MCP tools. ";
plan_prompt += "Inspect existing Content/ maps, Blueprints, and C++ under D:/code/omoba/omfue before planning. ";
plan_prompt += "Use read_file, grep, list_dir, and uecp MCP tools. Do not answer from memory. ";
plan_prompt += "Prefer Blueprints and existing omfue/OmRuntime patterns over new engine plugins. ";
plan_prompt += "Each task needs id, title, and done_when (an observable editor/asset check). ";
plan_prompt += "Goal (JSON): " + goal_json;
let planned = agent(plan_prompt, #{
    label: "plan-unreal",
    capability_mode: "all",
    output_schema: plan_schema,
});
let plan_summary = "";
let plan_json = "{}";
if planned != () && planned.success && planned.output != () {
    if planned.output.summary != () { plan_summary = planned.output.summary; }
    plan_json = json_encode(planned.output);
}
log("plan=" + plan_summary);

phase("Implement");
let impl_prompt = "";
impl_prompt += "Implement this Unreal plan in the LIVE editor using UECP MCP tools. ";
impl_prompt += "Do not wait for a human. Auto-approve destructive edits. Save assets. ";
impl_prompt += "Work only under D:/code/omoba/omfue. Do not commit git. ";
impl_prompt += "If a tool fails, retry once with a smaller step, then continue remaining tasks. ";
impl_prompt += "Use uecp MCP tools for Blueprint/actor/level edits. Use file tools only for C++/config. ";
impl_prompt += "If MCP is down, report that clearly instead of inventing success. ";
impl_prompt += "Goal (JSON): " + goal_json + "\n";
impl_prompt += "Plan (JSON): " + plan_json;
let implemented = agent(impl_prompt, #{
    label: "implement-unreal",
    capability_mode: "all",
});
let impl_ok = implemented != () && implemented.success;
log("implement_ok=" + json_encode(impl_ok));

phase("Verify");
let verify_prompt = "";
verify_prompt += "Adversarially verify the Unreal work actually landed. ";
verify_prompt += "Use uecp MCP tools and read_file/list_dir on D:/code/omoba/omfue. ";
verify_prompt += "Set passed=true only with concrete evidence (asset paths, actor names, compile result). ";
verify_prompt += "If you cannot inspect the editor, passed=false. ";
verify_prompt += "Goal (JSON): " + goal_json + "\n";
verify_prompt += "Plan (JSON): " + plan_json;
let verified = agent(verify_prompt, #{
    label: "verify-unreal",
    capability_mode: "all",
    output_schema: verify_schema,
});
let passed = false;
let evidence = "";
let gaps = "";
if verified != () && verified.success && verified.output != () {
    if verified.output.passed == true { passed = true; }
    if verified.output.evidence != () { evidence = verified.output.evidence; }
    if verified.output.gaps != () { gaps = verified.output.gaps; }
}

let report = "";
report += "# ue-ai-build\n\n";
report += "Goal: " + goal_json + "\n\n";
report += "MCP ready: " + json_encode(mcp_ready) + "\n\n";
report += "Plan: " + plan_summary + "\n\n";
report += "Implement ok: " + json_encode(impl_ok) + "\n\n";
report += "Passed: " + json_encode(passed) + "\n\n";
report += "Evidence: " + evidence + "\n\n";
report += "Gaps: " + gaps + "\n";
let report_path = write_scratch_file("report.md", report);

complete(#{
    path: report_path,
    passed: passed,
    summary: plan_summary,
    gaps: gaps,
    mcp_ready: mcp_ready,
});
