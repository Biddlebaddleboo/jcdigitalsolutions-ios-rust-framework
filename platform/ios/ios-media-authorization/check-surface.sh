#!/bin/sh
set -eu

if rg -n 'requestAccessForMediaType|requestRecordPermission|AVCaptureSession|AVCaptureDeviceInput|AVCapturePhotoOutput|AVCaptureAudioDataOutput|AVAudioEngine|AVAudioSession|AVCaptureDevice::devices|devicesWithMediaType|defaultDeviceWithMediaType|deviceWithUniqueID|startRunning|CMSampleBuffer|AVCaptureVideoDataOutput' platform/ios/ios-media-authorization/src; then
    printf '%s\n' 'capture or permission-request API found in status-only adapter' >&2
    exit 1
fi

printf '%s\n' 'status-only AVFoundation surface check passed'
