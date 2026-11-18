#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Every data-driven phone tool must survive the translation inventory."""
import importlib.util
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("locales", ROOT / "scripts/locales.py")
locales = importlib.util.module_from_spec(spec)
spec.loader.exec_module(locales)


class MobileToolLabels(unittest.TestCase):
    def test_tool_keys_are_inventoried_and_resolve_to_words(self):
        source = (ROOT / "src/crates/concat/src/studio/tool_layout.rs").read_text()
        keys = {key for key in re.findall(r'"([a-z][A-Za-z0-9]*(?:\.[a-z][A-Za-z0-9]*)+)"', source)}
        self.assertTrue({"phone.captions", "phone.voiceover", "phone.speak"} <= keys)
        self.assertLessEqual(keys, locales.code_keys())
        english = json.loads((locales.LOCALES / "en.json").read_text())
        for key in keys:
            with self.subTest(key=key):
                self.assertIn(key, english)
                self.assertNotEqual(english[key], key)
                self.assertTrue(english[key].strip())


if __name__ == "__main__":
    unittest.main()
