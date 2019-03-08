/*
 *  Copyright (c) 2016-present, Facebook, Inc.
 *  All rights reserved.
 *
 *  This source code is licensed under the BSD-style license found in the
 *  LICENSE file in the root directory of this source tree. An additional grant
 *  of patent rights can be found in the PATENTS file in the same directory.
 *
 */

namespace cpp2 facebook.eden

// A list of takeover data serialization versions that the client supports
struct TakeoverVersionQuery {
  // The set of versions supported by the client
  1: set<i32> versions,
}

struct SerializedInodeMapEntry {
  1: i64 inodeNumber,
  2: i64 parentInode,
  3: string name,
  4: bool isUnlinked,
  5: i64 numFuseReferences,
  6: string hash,
  7: i32 mode,
}

struct SerializedInodeMap {
  2: list<SerializedInodeMapEntry> unloadedInodes,
}

struct SerializedFileHandleMap {}

struct SerializedMountInfo {
  1: string mountPath,
  2: string stateDirectory,
  3: list<string> bindMountPaths,
  // This binary blob is really a fuse_init_out instance.
  // We don't transcribe that into thrift because the layout
  // is system dependent and happens to be flat and thus is
  // suitable for reinterpret_cast to be used upon it to
  // access the struct once we've moved it across the process
  // boundary.  Note that takeover is always local to the same
  // machine and thus has the same endianness.
  4: binary connInfo, // fuse_init_out

  // Removed, do not use 5
  // 5: SerializedFileHandleMap fileHandleMap,

  6: SerializedInodeMap inodeMap,
}

union SerializedTakeoverData {
  1: list<SerializedMountInfo> mounts,
  2: string errorReason,
}
