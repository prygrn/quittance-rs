Write tests that express the intention and expected behavior of the code.
A newly written test must fail before the code that satisfies it is written.
Keep each test minimal while still representative of the behavior it guarantees.
Write only the code necessary to make the failing test pass.
Do not add code beyond what the current test requires.
After a test passes, you may propose a refactor.
A refactor must not change the code's observable behavior.
Justify any proposed refactor.
Run the test suite after each step and confirm it passes.
Do not modify code in a way that circumvents existing tests.
When creating a new class, module, file, or namespace, first write a test asserting its structure.
Every behavior an implementation adds is covered by a test.
Use setup hooks to prepare shared test state.
Use teardown hooks to clean up shared test state.
Keep tests independent of each other.
Keep each test focused on one behavior.
Write tests in English.
Structure tests using the Arrange-Act-Assert pattern.
Test user-facing behavior through the actions a user would take, not implementation details.
Measure and log request-to-response latency in API endpoint tests if applicable.
Specify the expected output data for each integration test.
Verify data transformation as it flows through each component under test.
Mock external API calls in integration tests.
Mock external data results with realistic, accurate data.
Assert that mocked external calls receive valid data.
Verify that asynchronous commands are dispatched correctly.
Assert that asynchronous operations emit the correct results.
Keep unit tests fast.
Test edge cases.
Test error-handling code paths.
Avoid brittle assertions such as matching exact error message text.
Test all business logic — any function whose behavior can regress without the compiler catching it.
Test pure functions without any setup.
When the full test suite is too slow to run, run only the tests scoped to the feature.
Write tests for new code.
