# iOS PCM audio output

`ios-pcm-audio-output` feeds interleaved Int16 or Float32 PCM into a public `AVAudioSourceNode` connected to an `AVAudioEngine` main mixer. Configure the finite positive sample rate and one or two channels explicitly, then create a bounded queue and copy whole frames with `write_int16_interleaved` or `write_float32_interleaved`

Writes are all-or-nothing. The crate copies caller samples synchronously into its preallocated queue, does not retain the caller's slice, and returns `QueueFull` without a partial write when capacity is insufficient. The render callback allocates nothing, takes no locks, invokes no app closure, and fills any underrun with silence. Stop is terminal and idempotent; it stops the engine and detaches the source node. Any queued frames remain bounded and retained until drop, but are not rendered after stop. Drop performs the same cleanup

`AVAudioSourceNode` requires iOS 13.0. The host must configure and activate its own `AVAudioSession` category, mode, and route policy before `start`. The crate never requests permission and never calls a session setter or activation method. Apple documents that AVAudioEngine preparation/start may implicitly activate the shared session, so the host must account for that native side effect. Stop does not deactivate the session

The device and Simulator link probes check API imports and deployment metadata but do not execute. Host queue tests establish bounded-copy, ordering, sample preservation, frame validation, and overflow behavior; they do not prove audible output, output route selection, interruption recovery, timing, or physical-device behavior
