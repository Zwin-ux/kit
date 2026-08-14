import { sessionId } from "./auth";

test("session", () => {
  expect(sessionId()).toBe("s");
});
