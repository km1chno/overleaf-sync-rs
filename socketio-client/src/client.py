# This solution is taken from https://github.com/zzjjzzgggg/overleaf-sync/blob/887fdd8e5709ca3f02fc29973ece84dbfe6f9430/olsync/olsync/olclient.py#L136

import time
import argparse
from socketIO_client import SocketIO

BASE_URL = "https://www.overleaf.com"


def main():
    parser = argparse.ArgumentParser(description="overleaf-sync-rs socket.io client")

    parser.add_argument("GCLB", help="GCLB cookie value.")
    parser.add_argument("overleaf_session2", help="overleaf_session2 cookie value.")
    parser.add_argument("project_id", help="The id of Overleaf project for which data is fetched.")

    args = parser.parse_args()

    GCLB = args.GCLB
    overleaf_session_2 = args.overleaf_session2
    project_id = args.project_id

    project_infos = None

    def set_project_infos(project_infos_dict):
        nonlocal project_infos
        project_infos = project_infos_dict.get("project", {})

    cookie = "GCLB={}; overleaf_session2={}".format(GCLB, overleaf_session_2)

    socket_io = SocketIO(BASE_URL,
                         params={
                             't': int(time.time()),
                             'projectId': project_id
                         },
                         headers={'Cookie': cookie, 'referer': f"{BASE_URL}/project/{project_id}"})

    socket_io.on('connect', lambda: None)
    socket_io.wait_for_callbacks()

    socket_io.on('joinProjectResponse', set_project_infos)

    while project_infos is None:
        socket_io.wait(1)

    if socket_io.connected:
        socket_io.disconnect()

    print(project_infos)

if __name__ == "__main__":
    main()
