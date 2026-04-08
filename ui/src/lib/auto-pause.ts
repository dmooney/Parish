/**
 * Auto-pause idle tracker.
 *
 * Listens for mouse/keyboard activity and, after `idleMs` of no activity,
 * issues a `/pause` command. On the next activity event, if WE were the
 * ones who auto-paused, issues `/resume`. Manual pauses (via slash command
 * or any path that doesn't go through our `pausedByAutoIdle` flag) are
 * sticky — we never auto-resume them.
 *
 * Extracted as a plain TS module so it can be unit-tested without rendering
 * the SvelteKit page.
 */

export interface AutoPauseDeps {
	/** How long the player must be idle (real-time ms) before auto-pause. */
	readonly idleMs: number;
	/** Throttle window for mousemove events. Mousemove only counts as
	 *  activity once per this many ms to avoid resetting the timer constantly. */
	readonly mousemoveThrottleMs: number;
	/** Function to call when auto-pausing. Receives the slash command text. */
	submitInput: (text: string) => void | Promise<void>;
	/** Returns whether the game world is currently paused (any source). */
	isWorldPaused: () => boolean;
	/** Schedules a callback after the given ms. Defaults to `setTimeout`. */
	setTimeout?: (fn: () => void, ms: number) => ReturnType<typeof setTimeout>;
	/** Cancels a previously-scheduled callback. Defaults to `clearTimeout`. */
	clearTimeout?: (handle: ReturnType<typeof setTimeout>) => void;
	/** Returns the current wall-clock ms. Defaults to `Date.now`. */
	now?: () => number;
}

export interface AutoPauseTracker {
	/** Call on user activity. Non-throttled — use this for keydown etc. */
	recordActivity: () => void;
	/** Call on mousemove. Internally throttled to once per `mousemoveThrottleMs`. */
	recordMousemove: () => void;
	/** Call when the world snapshot updates so we can clear stale auto-pause state. */
	onWorldStateChange: (paused: boolean) => void;
	/** Tear down internal timers. */
	dispose: () => void;
	/** Whether the most recent pause was triggered by us (for tests). */
	wasAutoPaused: () => boolean;
}

export function createAutoPauseTracker(deps: AutoPauseDeps): AutoPauseTracker {
	const setT = deps.setTimeout ?? setTimeout;
	const clearT = deps.clearTimeout ?? clearTimeout;
	const now = deps.now ?? Date.now;

	let pausedByAutoIdle = false;
	let idleTimer: ReturnType<typeof setTimeout> | null = null;
	let lastMousemoveAt = 0;
	let disposed = false;

	function scheduleAutoPause() {
		if (disposed) return;
		if (idleTimer !== null) clearT(idleTimer);
		idleTimer = setT(() => {
			if (disposed) return;
			// Only auto-pause if not already paused — otherwise we'd
			// shadow a manual /pause as if we'd done it.
			if (!deps.isWorldPaused()) {
				pausedByAutoIdle = true;
				void deps.submitInput('/pause');
			}
		}, deps.idleMs);
	}

	function recordActivity() {
		if (disposed) return;
		// If the previous pause was ours, resume. Manual pauses stay paused.
		if (pausedByAutoIdle && deps.isWorldPaused()) {
			pausedByAutoIdle = false;
			void deps.submitInput('/resume');
		}
		// If the world is no longer paused but we were tracking auto-pause,
		// the user manually resumed — clear our flag.
		if (pausedByAutoIdle && !deps.isWorldPaused()) {
			pausedByAutoIdle = false;
		}
		scheduleAutoPause();
	}

	function recordMousemove() {
		if (disposed) return;
		const t = now();
		if (t - lastMousemoveAt < deps.mousemoveThrottleMs) return;
		lastMousemoveAt = t;
		recordActivity();
	}

	function onWorldStateChange(paused: boolean) {
		// If the world flips from paused → unpaused via any path other than
		// our auto-resume, drop our flag so we don't shadow it later.
		if (!paused && pausedByAutoIdle) {
			pausedByAutoIdle = false;
		}
	}

	function dispose() {
		disposed = true;
		if (idleTimer !== null) {
			clearT(idleTimer);
			idleTimer = null;
		}
	}

	// Start the initial countdown
	scheduleAutoPause();

	return {
		recordActivity,
		recordMousemove,
		onWorldStateChange,
		dispose,
		wasAutoPaused: () => pausedByAutoIdle
	};
}
