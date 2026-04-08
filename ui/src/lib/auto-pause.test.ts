import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createAutoPauseTracker } from './auto-pause';

describe('createAutoPauseTracker', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	function makeTracker(opts: {
		idleMs?: number;
		mousemoveThrottleMs?: number;
		isPaused?: () => boolean;
	} = {}) {
		const submitInput = vi.fn(async (_text: string) => {});
		let paused = false;
		const isPaused = opts.isPaused ?? (() => paused);
		const tracker = createAutoPauseTracker({
			idleMs: opts.idleMs ?? 60_000,
			mousemoveThrottleMs: opts.mousemoveThrottleMs ?? 1000,
			submitInput,
			isWorldPaused: isPaused
		});
		return {
			submitInput,
			tracker,
			setPaused: (v: boolean) => {
				paused = v;
			}
		};
	}

	it('auto-pauses after idleMs of no activity', () => {
		const { submitInput } = makeTracker();
		expect(submitInput).not.toHaveBeenCalled();
		vi.advanceTimersByTime(60_000);
		expect(submitInput).toHaveBeenCalledTimes(1);
		expect(submitInput).toHaveBeenCalledWith('/pause');
	});

	it('does not auto-pause if activity occurs before idleMs', () => {
		const { submitInput, tracker } = makeTracker();
		vi.advanceTimersByTime(30_000);
		tracker.recordActivity();
		vi.advanceTimersByTime(30_000);
		expect(submitInput).not.toHaveBeenCalled();
		// And another full window from the activity → should fire
		vi.advanceTimersByTime(30_000);
		expect(submitInput).toHaveBeenCalledTimes(1);
		expect(submitInput).toHaveBeenCalledWith('/pause');
	});

	it('auto-resumes when activity occurs after auto-pause', () => {
		const { submitInput, tracker, setPaused } = makeTracker();
		// Trigger auto-pause
		vi.advanceTimersByTime(60_000);
		expect(submitInput).toHaveBeenCalledWith('/pause');
		// Pretend the backend processed the /pause
		setPaused(true);
		// Player moves
		tracker.recordActivity();
		expect(submitInput).toHaveBeenCalledTimes(2);
		expect(submitInput).toHaveBeenLastCalledWith('/resume');
	});

	it('does NOT auto-resume manual pauses', () => {
		const { submitInput, tracker, setPaused } = makeTracker();
		// User manually pauses (not via auto-idle)
		setPaused(true);
		// Idle for 60s — we should NOT auto-pause again (it's already paused)
		vi.advanceTimersByTime(60_000);
		expect(submitInput).not.toHaveBeenCalled();
		// Player moves — we should NOT auto-resume (we never auto-paused)
		tracker.recordActivity();
		expect(submitInput).not.toHaveBeenCalled();
	});

	it('throttles mousemove events', () => {
		const { submitInput, tracker } = makeTracker({ mousemoveThrottleMs: 1000 });
		// Many rapid mousemoves within a second should only count as one activity
		for (let i = 0; i < 50; i++) {
			tracker.recordMousemove();
		}
		// Idle for almost the full window
		vi.advanceTimersByTime(59_999);
		expect(submitInput).not.toHaveBeenCalled();
		// One more ms and the auto-pause fires
		vi.advanceTimersByTime(1);
		expect(submitInput).toHaveBeenCalledTimes(1);
	});

	it('mousemove after throttle window resets the timer', () => {
		const { submitInput, tracker } = makeTracker({ mousemoveThrottleMs: 1000 });
		// Initial mousemove
		tracker.recordMousemove();
		vi.advanceTimersByTime(30_000);
		// 30s later, mousemove again — past the throttle window, counts as activity
		tracker.recordMousemove();
		vi.advanceTimersByTime(30_000);
		// Total elapsed since the second mousemove is 30s, so no pause yet
		expect(submitInput).not.toHaveBeenCalled();
		vi.advanceTimersByTime(30_000);
		expect(submitInput).toHaveBeenCalledWith('/pause');
	});

	it('dispose clears the pending timer', () => {
		const { submitInput, tracker } = makeTracker();
		tracker.dispose();
		vi.advanceTimersByTime(60_000);
		expect(submitInput).not.toHaveBeenCalled();
	});

	it('onWorldStateChange clears auto-pause flag if user manually resumes', () => {
		const { submitInput, tracker, setPaused } = makeTracker();
		// Trigger auto-pause
		vi.advanceTimersByTime(60_000);
		setPaused(true);
		// User runs /resume manually somehow → world snapshot updates
		setPaused(false);
		tracker.onWorldStateChange(false);
		expect(tracker.wasAutoPaused()).toBe(false);
		// Now player moves — should NOT issue /resume (we already cleared the flag)
		// But it WILL re-schedule auto-pause for the next idle window.
		submitInput.mockClear();
		tracker.recordActivity();
		expect(submitInput).not.toHaveBeenCalledWith('/resume');
	});

	it('wasAutoPaused returns true between auto-pause and resume', () => {
		const { tracker, setPaused } = makeTracker();
		expect(tracker.wasAutoPaused()).toBe(false);
		vi.advanceTimersByTime(60_000);
		expect(tracker.wasAutoPaused()).toBe(true);
		setPaused(true);
		tracker.recordActivity();
		expect(tracker.wasAutoPaused()).toBe(false);
	});
});
