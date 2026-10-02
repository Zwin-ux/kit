import { mock, test, type TestBody } from 'claude-code/testing'

// Exercise the same successful registration path as the native host before UI/commands.
// Motion tests keep their own startup, clock and environment fixtures instead.
export function readyTest(name: string, body: (...args: [...Parameters<TestBody>, ready: () => Promise<void>]) => ReturnType<TestBody>): void {
  test(name, async ($, on) => {
    mock.env(on, { KIT_MOTION: 'off' })
    on('command.register', () => ({ value: { command: 'kit' } }))
    on('session.start', () => ({ cwd: '/work' }))
    const ready = async () => {
      await $.session.start({ surface: 'terminal', isInteractive: true, cwd: '/work' })
    }
    return body($, on, ready)
  })
}
