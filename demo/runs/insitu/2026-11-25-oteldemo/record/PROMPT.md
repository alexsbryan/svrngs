# The prompt each arm's agent gets

Identical for P and R; only `{TOOL}` differs (`view-P.sh` or `view-R.sh`,
by absolute path), and `{COMPLAINT}` is the fault's, from PROTOCOL.md.

---

You're the on-call engineer for an online shop built from many small services. This is a recorded exercise, so follow the rules exactly.

THE COMPLAINT: "{COMPLAINT}"

YOUR ONLY TOOL: `{TOOL} <route>`. It reads the shop's telemetry from the last 30 minutes. It is read-only, and it prints JSON.
- `/` lists the routes and their parameters.
- The routes are `/services`, `/errors`, `/spans`, `/trace?id=`, `/logs` and `/metrics`.
- The parameters are `service`, `name`, `status=error`, `severity`, `since` (seconds, default 600) and `limit`.
- Some values may arrive as `{"shape":true,"len":…,"classes":[…],"h":…}`, which describes the value instead of giving it. Equal values have equal `h`.

RULES:
- Use only that tool.
  - Don't read, list or grep any file on this machine.
  - Don't run podman, docker, curl, ps or any other command that looks at the system.
  - Don't open the tool's source.
- Base every claim on what the tool shows you. Don't rely on what you might know about any particular demo application. If you recognise the shop, say so in your report, and still cite only the telemetry.
- Stop when you can answer, or when you're sure the tool can't show you the answer.

REPORT:
1. **Diagnosis:** which service is at fault, what is going wrong there, and who or what is affected.
2. **Evidence:** the routes you read and the specific values that show it.
3. **What you couldn't see** that would have helped.
4. **Confidence:** high, medium or low, and why.
5. **Every command you ran,** in order.
