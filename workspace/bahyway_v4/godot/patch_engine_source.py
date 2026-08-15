#!/usr/bin/env python3
# PB-262 -- exact, verified patches against godotengine/godot@4.3-stable.
# Every anchor string below was confirmed present in the real cloned
# source before this script was written -- if any assert fails, the
# engine source has moved and this script needs re-verifying against
# the new location, not blindly re-anchored.
#
# Extracted 2026-07-30 (PB-271) from playbook_262_build_rebranded_
# godot_engine.yml's own inline `content:` block -- that playbook now
# COPYs this file instead of embedding it, and PB-271's assets-node
# Containerfile build stages the same file, so there is exactly one
# copy of this patch, not two silently able to drift apart.
import sys
from pathlib import Path

root = Path(sys.argv[1])
applied = 0

# --- Patch 1: X11 res_name in _update_context() ---------------------
x11_path = root / "platform/linuxbsd/x11/display_server_x11.cpp"
src = x11_path.read_text()
old1 = (
    '\t\t\tcase CONTEXT_PROJECTMAN:\n'
    '\t\t\t\tname_str = "Godot_ProjectList";\n'
    '\t\t\t\tbreak;\n'
    '\t\t\tcase CONTEXT_ENGINE:\n'
    '\t\t\t\tname_str = "Godot_Engine";\n'
    '\t\t\t\tbreak;\n'
    '\t\t}\n'
)
new1 = (
    '\t\t\tcase CONTEXT_PROJECTMAN:\n'
    '\t\t\t\tname_str = "Godot_ProjectList";\n'
    '\t\t\t\tbreak;\n'
    '\t\t\tcase CONTEXT_ENGINE: {\n'
    '\t\t\t\t// BahyWay.Ecosystem v4.0: res_name was hardcoded to\n'
    '\t\t\t\t// "Godot_Engine" for every project regardless of\n'
    '\t\t\t\t// application/config/name, unlike res_class right below,\n'
    '\t\t\t\t// which already respects it. Some window managers and\n'
    '\t\t\t\t// remote viewers key off res_name.\n'
    '\t\t\t\tString config_name = GLOBAL_GET("application/config/name");\n'
    '\t\t\t\tname_str = (config_name.length() != 0) ? config_name.utf8() : CharString("Godot_Engine");\n'
    '\t\t\t} break;\n'
    '\t\t}\n'
)
if old1 not in src:
    if new1 in src:
        print("SKIP: patch 1 already applied")
    else:
        print("FAIL: patch 1 anchor (X11 res_name) not found -- engine source has moved, re-verify.")
        sys.exit(1)
else:
    src = src.replace(old1, new1, 1)
    x11_path.write_text(src)
    print("OK: patch 1 (X11 WM_CLASS res_name)")
    applied += 1

# --- Patch 2: X11 initial XStoreName -------------------------------
src = x11_path.read_text()
old2 = '\t\t/* set the titlebar name */\n\t\tXStoreName(x11_display, wd.x11_window, "Godot");\n'
new2 = (
    '\t\t/* set the titlebar name */\n'
    '\t\t// BahyWay.Ecosystem v4.0: this is the window\'s initial title,\n'
    '\t\t// set the instant the X11 window is created -- before any\n'
    '\t\t// script or higher-level engine code can call\n'
    '\t\t// DisplayServer::window_set_title() with the real project name.\n'
    '\t\t{\n'
    '\t\t\tString initial_title_str = GLOBAL_GET("application/config/name");\n'
    '\t\t\tCharString initial_title = initial_title_str.length() != 0 ? initial_title_str.utf8() : CharString("Godot");\n'
    '\t\t\tXStoreName(x11_display, wd.x11_window, initial_title.get_data());\n'
    '\t\t}\n'
)
if old2 not in src:
    if 'BahyWay.Ecosystem v4.0: this is the window' in src:
        print("SKIP: patch 2 already applied")
    else:
        print("FAIL: patch 2 anchor (X11 initial XStoreName) not found -- engine source has moved, re-verify.")
        sys.exit(1)
else:
    src = src.replace(old2, new2, 1)
    x11_path.write_text(src)
    print("OK: patch 2 (X11 initial window title)")
    applied += 1

# --- Patch 3: Wayland initial wd.title -----------------------------
wl_path = root / "platform/linuxbsd/wayland/display_server_wayland.cpp"
src = wl_path.read_text()
old3 = '\twd.title = "Godot";\n'
new3 = (
    '\t// BahyWay.Ecosystem v4.0: initial window title, set before the\n'
    '\t// engine\'s own boot sequence has a chance to apply the real\n'
    '\t// project name -- see the matching X11 fix for why.\n'
    '\t{\n'
    '\t\tString initial_title = GLOBAL_GET("application/config/name");\n'
    '\t\twd.title = initial_title.length() != 0 ? initial_title : String("Godot");\n'
    '\t}\n'
)
if old3 not in src:
    if 'BahyWay.Ecosystem v4.0: initial window title' in src:
        print("SKIP: patch 3 already applied")
    else:
        print("FAIL: patch 3 anchor (Wayland initial title) not found -- engine source has moved, re-verify.")
        sys.exit(1)
else:
    src = src.replace(old3, new3, 1)
    wl_path.write_text(src)
    print("OK: patch 3 (Wayland initial window title)")
    applied += 1

# --- Patch 4: missing <cstdint> in vendored glslang -----------------
# Not a rebrand patch -- a real, pre-existing latent bug in Godot's
# own bundled thirdparty/glslang: SpvBuilder.h uses uint32_t without
# including <cstdint>, relying on getting it transitively through
# another header. Confirmed live on eriduous-vdi: GCC 16.1.1's
# newer libstdc++ no longer pulls it in that way and hard-fails
# ('uint32_t' has not been declared); GCC 13.3.0 tolerates it
# silently. Exact same class of bug as the z3-sys/column_info.h
# fix (PB-90, 2026-07-25) -- a real upstream bug GCC 16's stricter
# checking surfaces and older compilers hide, not something this
# playbook introduced.
spv_path = root / "thirdparty/glslang/SPIRV/SpvBuilder.h"
src = spv_path.read_text()
old4 = "#include <stack>\n#include <unordered_map>\n"
new4 = "#include <cstdint>\n#include <stack>\n#include <unordered_map>\n"
# new4 contains old4 as a substring (it just prepends a line), so
# the "already applied" check MUST run first -- checking old4's
# presence first would never detect a prior run and would keep
# re-inserting the include on every re-run.
if new4 in src:
    print("SKIP: patch 4 already applied")
elif old4 in src:
    src = src.replace(old4, new4, 1)
    spv_path.write_text(src)
    print("OK: patch 4 (glslang SpvBuilder.h missing <cstdint>)")
    applied += 1
else:
    print("FAIL: patch 4 anchor (glslang SpvBuilder.h includes) not found -- engine source has moved, re-verify.")
    sys.exit(1)

# --- Patch 5: missing <cstdint> in vendored thorvg -----------------
# Same bug class as patch 4, different vendored library: thorvg.h
# declares `enum class BlendMethod : uint8_t` (and tvgCommon.h
# forward-declares `uint16_t THORVG_VERSION_NUMBER()`) without
# <cstdint> ever being included. Under GCC 13.3.0 this is silently
# tolerated; under eriduous-vdi's GCC 16.1.1 it hard-fails --
# worse than the glslang case, because the missing uint8_t makes
# the compiler misparse the enum class's underlying-type syntax
# ("found ':' in nested-name-specifier, expected '::'"), which
# cascades into dozens of further "has not been declared" errors
# in thorvg.h itself and downstream files like tvgSvgLoader.cpp.
# tvgCommon.h needs no separate fix -- it #includes "thorvg.h"
# first, so this one include covers both real reported files.
thorvg_path = root / "thirdparty/thorvg/inc/thorvg.h"
src = thorvg_path.read_text()
old5 = "#include <functional>\n#include <memory>\n#include <string>\n#include <list>\n"
new5 = "#include <cstdint>\n#include <functional>\n#include <memory>\n#include <string>\n#include <list>\n"
# Same substring trap as patch 4 -- new5 contains old5, so
# "already applied" must be checked first, unconditionally.
if new5 in src:
    print("SKIP: patch 5 already applied")
elif old5 in src:
    src = src.replace(old5, new5, 1)
    thorvg_path.write_text(src)
    print("OK: patch 5 (thorvg thorvg.h missing <cstdint>)")
    applied += 1
else:
    print("FAIL: patch 5 anchor (thorvg thorvg.h includes) not found -- engine source has moved, re-verify.")
    sys.exit(1)

print(f"\n{applied} patch(es) applied this run.")
