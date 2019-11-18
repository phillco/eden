# Portions Copyright (c) Facebook, Inc. and its affiliates.
#
# This software may be used and distributed according to the terms of the
# GNU General Public License version 2.

# policy.py - module policy logic for Mercurial.
#
# Copyright 2015 Gregory Szorc <gregory.szorc@gmail.com>
#
# This software may be used and distributed according to the terms of the
# GNU General Public License version 2 or any later version.

from __future__ import absolute_import

import os
import sys


# Rules for how modules can be loaded. Values are:
#
#    c - require C extensions
#    allow - allow pure Python implementation when C loading fails
#    cffi - required cffi versions (implemented within pure module)
#    cffi-allow - allow pure Python implementation if cffi version is missing
#    py - only load pure Python modules
#
# By default, fall back to the pure modules so the in-place build can
# run without recompiling the C extensions. This will be overridden by
# __modulepolicy__ generated by setup.py.
policy = b"allow"
_packageprefs = {
    # policy: (versioned package, pure package)
    b"c": (r"cext", None),
    b"allow": (r"cext", r"pure"),
    b"cffi": (r"cffi", None),
    b"cffi-allow": (r"cffi", r"pure"),
    b"py": (None, r"pure"),
}

try:
    from . import __modulepolicy__

    policy = __modulepolicy__.modulepolicy
except ImportError:
    pass

# PyPy doesn't load C extensions.
#
# The canonical way to do this is to test platform.python_implementation().
# But we don't import platform and don't bloat for it here.
if r"__pypy__" in sys.builtin_module_names:
    policy = b"cffi"

# Our C extensions aren't yet compatible with Python 3. So use pure Python
# on Python 3 for now.
if sys.version_info[0] >= 3:
    policy = b"py"

# Environment variable can always force settings.
if sys.version_info[0] >= 3:
    if r"HGMODULEPOLICY" in os.environ:
        policy = os.environ[r"HGMODULEPOLICY"].encode(r"utf-8")
else:
    policy = os.environ.get(r"HGMODULEPOLICY", policy)


def _importfrom(pkgname, modname):
    # from .<pkgname> import <modname> (where . is looked through this module)
    level = 1  # relative import (relative to ".")
    if pkgname == "cext":
        # ".cext" was moved to "edenscmnative"
        pkgname = "edenscmnative"
        level = 0  # use absolute import
    fakelocals = {}
    pkg = __import__(pkgname, globals(), fakelocals, [modname], level=level)
    try:
        fakelocals[modname] = mod = getattr(pkg, modname)
    except AttributeError:
        raise ImportError(r"cannot import name %s" % modname)
    # force import; fakelocals[modname] may be replaced with the real module
    getattr(mod, r"__doc__", None)
    return fakelocals[modname]


# keep in sync with "version" in C modules
_cextversions = {
    (r"cext", r"base85"): 1,
    (r"cext", r"bdiff"): 1,
    (r"cext", r"diffhelpers"): 1,
    (r"cext", r"mpatch"): 1,
    (r"cext", r"osutil"): 2,
    (r"cext", r"parsers"): 5,
}

# map import request to other package or module
_modredirects = {
    (r"cext", r"charencode"): (r"cext", r"parsers"),
    (r"cffi", r"base85"): (r"pure", r"base85"),
    (r"cffi", r"charencode"): (r"pure", r"charencode"),
    (r"cffi", r"diffhelpers"): (r"pure", r"diffhelpers"),
    (r"cffi", r"parsers"): (r"pure", r"parsers"),
}


def _checkmod(pkgname, modname, mod):
    expected = _cextversions.get((pkgname, modname))
    actual = getattr(mod, r"version", None)
    if actual != expected:
        raise ImportError(
            r"cannot import module %s.%s "
            r"(expected version: %d, actual: %r)" % (pkgname, modname, expected, actual)
        )


def importmod(modname):
    """Import module according to policy and check API version"""
    try:
        verpkg, purepkg = _packageprefs[policy]
    except KeyError:
        raise ImportError(r"invalid HGMODULEPOLICY %r" % policy)
    assert verpkg or purepkg
    if verpkg:
        pn, mn = _modredirects.get((verpkg, modname), (verpkg, modname))
        try:
            mod = _importfrom(pn, mn)
            if pn == verpkg:
                _checkmod(pn, mn, mod)
            return mod
        except ImportError:
            if not purepkg:
                raise
    pn, mn = _modredirects.get((purepkg, modname), (purepkg, modname))
    return _importfrom(pn, mn)