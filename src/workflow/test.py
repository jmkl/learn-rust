import json
import urllib.request

with open("workflow.json") as f:
    p = {"prompt":f.read()}
    print(str(json.dumps(p).encode("utf-8")))