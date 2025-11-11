Debounce for skip/previous and media controls

What changed

- A small debounce (150ms) was introduced for skip (Next) and previous (Previous) actions to prevent rapid UI clicks from flooding the player thread and triggering races.
- OS-level media control events are now routed through the GUI `PlaybackController` via a small channel. This allows the controller to apply the same debounce to hardware/OS media keys.

Why

Spamming skip/previous could previously cause rare race conditions or channel errors between the GUI and player thread. The debounce is a pragmatic mitigation that preserves normal UX while preventing flood-induced failures.

How to test

1. Start the app with debug logs:

```fish
env RUST_LOG=debug RUST_BACKTRACE=1 cargo run --bin psst-gui
```

2. Play a playlist or album (so the queue is non-empty).
3. Rapidly press the Next/Previous UI buttons or your keyboard/media keys. You should see `next skipped due to debounce` or `media control next/previous skipped due to debounce` in logs when events are within ~150 ms of each other.

Notes

- The debounce window is currently 150 ms. If you prefer a different value, edit `PlaybackController::should_throttle_skip()` in `psst-gui/src/controller/playback.rs`.
- This change is intentionally conservative. If the issue persists under heavy stress, we can add an automatic resume-on-restart or more in-depth fixes.
