import json
import subprocess
import sys

result = subprocess.run(
    ["gh", "api", "repos/Crazygiscool/Lexic/actions/runs/43783033900/jobs"],
    capture_output=True,
    text=True
)
d = json.loads(result.stdout)
for j in d.get("jobs", []):
    print(f"Job: {j.get('name')} id={j.get('id')} conclusion={j.get('conclusion')}")