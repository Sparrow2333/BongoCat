"""Verify the effective proxy deployment, including Compose port replacement."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import unittest


ROOT = Path(__file__).resolve().parents[2]
DOCKER = shutil.which("docker")


@unittest.skipUnless(DOCKER, "Docker CLI is required to validate the Compose merge")
class RoomComposeTests(unittest.TestCase):
    def configuration(self, port=None):
        env = os.environ.copy()
        env.pop("BONGOCAT_ROOM_PORT", None)
        env.pop("COMPOSE_PROJECT_NAME", None)
        if port is not None:
            env["BONGOCAT_ROOM_PORT"] = port
        result = subprocess.run(
            [
                DOCKER, "compose", "--env-file", os.devnull,
                "-f", "compose.yaml", "-f", "deploy/compose.caddy.yaml",
                "config", "--format", "json",
            ],
            cwd=ROOT, env=env, check=True, capture_output=True, text=True,
        )
        return json.loads(result.stdout)

    def assert_local_port(self, config, published):
        self.assertEqual(config["name"], "bongocat-room")
        room = config["services"]["room"]
        # An ordinary list merge would keep the base file's public port too.
        self.assertEqual(len(room["ports"]), 1)
        port = room["ports"][0]
        self.assertEqual(port["host_ip"], "127.0.0.1")
        self.assertEqual(port["target"], 8080)
        self.assertEqual(port["published"], published)
        self.assertEqual(port["protocol"], "tcp")
        self.assertEqual(room["restart"], "no")
        self.assertTrue(room["read_only"])
        self.assertEqual(room["cap_drop"], ["ALL"])

    def test_caddy_override_does_not_leave_a_public_port_or_second_supervisor(self):
        self.assert_local_port(self.configuration(), "18080")

    def test_custom_port_also_replaces_the_base_public_binding(self):
        self.assert_local_port(self.configuration("19080"), "19080")


class RoomSystemdTests(unittest.TestCase):
    @unittest.skipUnless(
        sys.platform == "linux" and shutil.which("systemd-analyze"),
        "systemd unit verification requires Linux with systemd-analyze",
    )
    def test_service_definition_is_valid_for_the_installed_systemd(self):
        if not Path("/usr/bin/docker").is_file():
            self.skipTest("the deployment requires Docker at /usr/bin/docker")
        subprocess.run(
            ["systemd-analyze", "verify", str(ROOT / "deploy/bongocat-room.service")],
            check=True, capture_output=True, text=True,
        )


if __name__ == "__main__":
    unittest.main()
