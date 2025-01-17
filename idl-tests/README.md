The "IDL tests" are unit tests for IDL parsing and recognition.

The test harness is  `gluegun-test-harness::idl_tests` which parses them with syn, generates the IDL, and then compares it against the `.idl` file found in the repository.

Differences are logged in a `.err` file and reported as errors.