/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

#pragma once

#include <folly/Conv.h>
#include <folly/File.h>
#include <folly/Range.h>
#include <folly/logging/LogLevel.h>
#include <memory>

namespace facebook {
namespace eden {

/**
 * StartupLogger provides an API for logging messages that should be displayed
 * to the user while edenfs is starting.
 *
 * If edenfs is daemonizing, the original foreground process will not exit until
 * success() or fail() is called.  Any messages logged with log() or warn() will
 * be shown printed in the original foreground process.
 */
class StartupLogger {
 public:
  StartupLogger() {} //= default;

  /**
   * daemonize the current process.
   *
   * This method returns in a new process.  This method will never return in the
   * parent process that originally called daemonize().  Instead the parent
   * waits for the child process to either call StartupLogger::success() or
   * StartupLogger::fail(), and exits with a status code based on which of these
   * was called.
   *
   * If logPath is non-empty the child process will redirect its stdout and
   * stderr file descriptors to the specified log file before returning.
   */
  void daemonize(folly::StringPiece logPath) {}

  /**
   * Log an informational message.
   */
  template <typename... Args>
  void log(Args&&... args) {
    // writeMessage(
    //    origStdout_,
    //    folly::LogLevel::DBG2,
    //    folly::to<std::string>(std::forward<Args>(args)...));
  }

  /**
   * Log a warning message.
   */
  template <typename... Args>
  void warn(Args&&... args) {
    // writeMessage(
    //    origStderr_,
    //    folly::LogLevel::WARN,
    //    folly::to<std::string>(std::forward<Args>(args)...));
  }

  /**
   * Indicate that startup has failed.
   *
   * This exits the current process, and also causes the original foreground
   * process to exit if edenfs has daemonized.
   */
  template <typename... Args>
  [[noreturn]] void exitUnsuccessfully(uint8_t exitCode, Args&&... args) {
    writeMessage(
        origStderr_,
        folly::LogLevel::ERR,
        folly::to<std::string>(std::forward<Args>(args)...));
    failAndExit(exitCode);
  }

  /**
   * Indicate that startup has succeeded.
   *
   * If edenfs has daemonized this will cause the original foreground edenfs
   * process to exit successfully.
   */
  void success() {}

 private:
  friend class StartupLoggerTest;

  using ResultType = uint8_t;

  struct ParentResult {
    template <typename... Args>
    explicit ParentResult(uint8_t code, Args&&... args)
        : exitCode(code),
          errorMessage(folly::to<std::string>(std::forward<Args>(args)...)) {}

    int exitCode;
    std::string errorMessage;
  };

  [[noreturn]] void failAndExit(uint8_t exitCode);

  std::optional<std::pair<pid_t, folly::File>> daemonizeImpl(
      folly::StringPiece logPath);

  /**
   * Create the pipe for communication between the parent process and the
   * daemonized child.  Stores the write end in pipe_ and returns the read end.
   */
  folly::File createPipe();

  [[noreturn]] void runParentProcess(
      folly::File readPipe,
      pid_t childPid,
      folly::StringPiece logPath);
  void prepareChildProcess(folly::StringPiece logPath);
  void redirectOutput(folly::StringPiece logPath);

  /**
   * Wait for the child process to write its initialization status.
   */
  ParentResult waitForChildStatus(
      const folly::File& pipe,
      pid_t childPid,
      folly::StringPiece logPath);
  ParentResult handleChildCrash(pid_t childPid);

  void writeMessage(
      const folly::File& file,
      folly::LogLevel level,
      folly::StringPiece message) {}
  void sendResult(ResultType result);

  // If stdout and stderr have been redirected during process daemonization,
  // origStdout_ and origStderr_ contain file descriptors referencing the
  // original stdout and stderr.  These are used to continue to print
  // informational messages directly to the user during startup even after
  // normal log redirection.
  //
  // If log redirection has not occurred these will simply be closed File
  // objects.  The normal logging mechanism is sufficient to show messages to
  // the user in this case.
  folly::File origStdout_;
  folly::File origStderr_;
  std::string logPath_;

  // If we have daemonized, pipe_ is a pipe connected to the original foreground
  // process.  We use this to inform the original process when we have fully
  // completed daemon startup.
  folly::File pipe_;
};

} // namespace eden
} // namespace facebook
