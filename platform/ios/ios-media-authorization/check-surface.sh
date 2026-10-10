#!/bin/sh
set -eu

if rg -n 'requestRecordPermission|AVCaptureSession|AVCaptureDeviceInput|AVCapturePhotoOutput|AVCaptureAudioDataOutput|AVAudioEngine|AVAudioSession|AVCaptureDevice::devices|devicesWithMediaType|defaultDeviceWithMediaType|deviceWithUniqueID|startRunning|CMSampleBuffer|AVCaptureVideoDataOutput' platform/ios/ios-media-authorization/src; then
    printf '%s\n' 'capture or audio-session API found in authorization adapter' >&2
    exit 1
fi

if ! rg -q 'requestAccessForMediaType_completionHandler' platform/ios/ios-media-authorization/src/request.rs; then
    printf '%s\n' 'AVFoundation authorization-request API is missing' >&2
    exit 1
fi

printf '%s\n' 'camera/microphone authorization surface check passed'
