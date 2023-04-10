#![allow(non_upper_case_globals)]

macro_rules! constant {
    ($name: ident) => {
        pub const $name: &str = stringify!($name);
    };
}

pub mod actions {
    constant!(OfxActionLoad);
    constant!(OfxActionOpenGLContextAttached);
    constant!(OfxActionOpenGLContextDetached);
    constant!(OfxActionDescribe);
    constant!(OfxActionUnload);
    constant!(OfxActionPurgeCaches);
    constant!(OfxActionSyncPrivateData);
    constant!(OfxActionCreateInstance);
    constant!(OfxActionDestroyInstance);
    constant!(OfxActionInstanceChanged);
    constant!(OfxActionBeginInstanceChanged);
    constant!(OfxActionEndInstanceChanged);
    constant!(OfxActionBeginInstanceEdit);
    constant!(OfxActionEndInstanceEdit);
}

pub mod suites {
    constant!(OfxMultiThreadSuite);
    constant!(OfxProgressSuite);
    constant!(OfxPropertySuite);
    constant!(OfxInteractSuite);
    constant!(OfxTimeLineSuite);
    constant!(OfxOpenGLRenderSuite);
    constant!(OfxImageEffectSuite);
    constant!(OfxMemorySuite);
    constant!(OfxParameterSuite);
    constant!(OfxParametricParameterSuite);
    constant!(OfxMessageSuite);
    constant!(OfxDialogSuite);
}

pub mod host {
    constant!(OfxParamHostPropMaxPages);
    constant!(OfxParamHostPropMaxParameters);
    constant!(OfxParamHostPropPageRowColumnCount);
    constant!(OfxParamHostPropSupportsBooleanAnimation);
    constant!(OfxParamHostPropSupportsChoiceAnimation);
    constant!(OfxParamHostPropSupportsCustomAnimation);
    constant!(OfxParamHostPropSupportsCustomInteract);
    constant!(OfxParamHostPropSupportsParametricAnimation);
    constant!(OfxParamHostPropSupportsStrChoiceAnimation);
    constant!(OfxParamHostPropSupportsStringAnimation);
    constant!(OfxImageEffectHostPropIsBackground);
    constant!(OfxImageEffectHostPropNativeOrigin);
    constant!(OfxImageEffectPluginPropHostFrameThreading);
    constant!(OfxPropHostOSHandle);
    constant!(OfxHostNativeOriginBottomLeft);
    constant!(OfxHostNativeOriginCenter);
    constant!(OfxHostNativeOriginTopLeft);
}

pub mod image_effect {
    constant!(OfxImageEffectActionBeginSequenceRender);
    constant!(OfxImageEffectActionDescribeInContext);
    constant!(OfxImageEffectActionEndSequenceRender);
    constant!(OfxImageEffectActionGetClipPreferences);
    constant!(OfxImageEffectActionGetFramesNeeded);
    constant!(OfxImageEffectActionGetRegionOfDefinition);
    constant!(OfxImageEffectActionGetRegionsOfInterest);
    constant!(OfxImageEffectActionGetTimeDomain);
    constant!(OfxImageEffectActionIsIdentity);
    constant!(OfxImageEffectActionRender);
    constant!(OfxImageEffectContextFilter);
    constant!(OfxImageEffectContextGeneral);
    constant!(OfxImageEffectContextGenerator);
    constant!(OfxImageEffectContextPaint);
    constant!(OfxImageEffectContextRetimer);
    constant!(OfxImageEffectContextTransition);
    constant!(OfxImageEffectFrameVarying);
    constant!(OfxImageEffectInstancePropEffectDuration);
    constant!(OfxImageEffectInstancePropSequentialRender);
    constant!(OfxImageEffectOutputClipName);
    constant!(OfxImageEffectPluginApi);
    constant!(OfxImageEffectPluginApiVersion);
    constant!(OfxImageEffectPluginPropFieldRenderTwiceAlways);
    constant!(OfxImageEffectPluginPropGrouping);
    constant!(OfxImageEffectPluginPropOverlayInteractV1);
    constant!(OfxImageEffectPluginPropSingleInstance);
    constant!(OfxImageEffectPluginRenderThreadSafety);
    constant!(OfxImageEffectPropClipPreferencesSlaveParam);
    constant!(OfxImageEffectPropComponents);
    constant!(OfxImageEffectPropContext);
    constant!(OfxImageEffectPropCudaEnabled);
    constant!(OfxImageEffectPropCudaRenderSupported);
    constant!(OfxImageEffectPropCudaStream);
    constant!(OfxImageEffectPropCudaStreamSupported);
    constant!(OfxImageEffectPropFieldToRender);
    constant!(OfxImageEffectPropFrameRange);
    constant!(OfxImageEffectPropFrameRate);
    constant!(OfxImageEffectPropFrameStep);
    constant!(OfxImageEffectPropInAnalysis);
    constant!(OfxImageEffectPropInteractiveRenderStatus);
    constant!(OfxImageEffectPropMetalCommandQueue);
    constant!(OfxImageEffectPropMetalEnabled);
    constant!(OfxImageEffectPropMetalRenderSupported);
    constant!(OfxImageEffectPropNoSpatialAwareness);
    constant!(OfxImageEffectPropOpenCLCommandQueue);
    constant!(OfxImageEffectPropOpenCLEnabled);
    constant!(OfxImageEffectPropOpenCLRenderSupported);
    constant!(OfxImageEffectPropOpenGLEnabled);
    constant!(OfxImageEffectPropOpenGLRenderSupported);
    constant!(OfxImageEffectPropOpenGLTextureIndex);
    constant!(OfxImageEffectPropOpenGLTextureTarget);
    constant!(OfxImageEffectPropPixelDepth);
    constant!(OfxImageEffectPropPluginHandle);
    constant!(OfxImageEffectPropPreMultiplication);
    constant!(OfxImageEffectPropProjectExtent);
    constant!(OfxImageEffectPropProjectOffset);
    constant!(OfxImageEffectPropProjectPixelAspectRatio);
    constant!(OfxImageEffectPropProjectSize);
    constant!(OfxImageEffectPropRegionOfDefinition);
    constant!(OfxImageEffectPropRegionOfInterest);
    constant!(OfxImageEffectPropRenderQualityDraft);
    constant!(OfxImageEffectPropRenderScale);
    constant!(OfxImageEffectPropRenderWindow);
    constant!(OfxImageEffectPropResolvePage);
    constant!(OfxImageEffectPropSequentialRenderStatus);
    constant!(OfxImageEffectPropSetableFielding);
    constant!(OfxImageEffectPropSetableFrameRate);
    constant!(OfxImageEffectPropSupportedComponents);
    constant!(OfxImageEffectPropSupportedContexts);
    constant!(OfxImageEffectPropSupportedPixelDepths);
    // Note: name and string value don't match
    pub const OfxImageEffectPropSupportsMultipleClipDepths: &str =
        "OfxImageEffectPropMultipleClipDepths";
    pub const OfxImageEffectPropSupportsMultipleClipPARs: &str =
        "OfxImageEffectPropMultipleClipPARs";
    constant!(OfxImageEffectPropSupportsMultiResolution);
    constant!(OfxImageEffectPropSupportsOverlays);
    constant!(OfxImageEffectPropSupportsTiles);
    constant!(OfxImageEffectPropTemporalClipAccess);
    constant!(OfxImageEffectPropUnmappedFrameRange);
    constant!(OfxImageEffectPropUnmappedFrameRate);
    constant!(OfxImageEffectRenderFullySafe);
    constant!(OfxImageEffectRenderInstanceSafe);
    constant!(OfxImageEffectRenderUnsafe);
    constant!(OfxImageEffectRetimerParamName);
    constant!(OfxImageEffectSimpleSourceClipName);
    constant!(OfxImageEffectTransitionParamName);
    constant!(OfxImageEffectTransitionSourceFromClipName);
    constant!(OfxImageEffectTransitionSourceToClipName);
    constant!(OfxImageClipPropConnected);
    constant!(OfxImageClipPropContinuousSamples);
    constant!(OfxImageClipPropFieldExtraction);
    constant!(OfxImageClipPropFieldOrder);
    constant!(OfxImageClipPropIsMask);
    constant!(OfxImageClipPropOptional);
    constant!(OfxImageClipPropThumbnail);
    constant!(OfxImageClipPropUnmappedComponents);
    constant!(OfxImageClipPropUnmappedPixelDepth);
    constant!(OfxImageComponentAlpha);
    constant!(OfxImageComponentNone);
    constant!(OfxImageComponentRGB);
    constant!(OfxImageComponentRGBA);
    constant!(OfxImageComponentYUVA);
    pub const OfxImageFieldBoth: &str = "OfxFieldBoth";
    pub const OfxImageFieldDoubled: &str = "OfxFieldDoubled";
    pub const OfxImageFieldLower: &str = "OfxFieldLower";
    pub const OfxImageFieldNone: &str = "OfxFieldNone";
    pub const OfxImageFieldSingle: &str = "OfxFieldSingle";
    pub const OfxImageFieldUpper: &str = "OfxFieldUpper";
    constant!(OfxImageOpaque);
    pub const OfxImagePreMultiplied: &str = "OfxImageAlphaPremultiplied";
    pub const OfxImageUnPreMultiplied: &str = "OfxImageAlphaUnPremultiplied";
    constant!(OfxImagePropBounds);
    constant!(OfxImagePropData);
    constant!(OfxImagePropField);
    constant!(OfxImagePropPixelAspectRatio);
    constant!(OfxImagePropRegionOfDefinition);
    constant!(OfxImagePropRowBytes);
    constant!(OfxImagePropUniqueIdentifier);
}

pub mod properties {
    constant!(OfxPropAPIVersion);
    constant!(OfxPropChangeReason);
    constant!(OfxPropEffectInstance);
    constant!(OfxPropIcon);
    constant!(OfxPropInstanceData);
    constant!(OfxPropIsInteractive);
    constant!(OfxPropKeyString);
    constant!(OfxPropKeySym);
    constant!(OfxPropLabel);
    constant!(OfxPropLongLabel);
    constant!(OfxPropName);
    constant!(OfxPropParamSetNeedsSyncing);
    constant!(OfxPropPluginDescription);
    constant!(OfxPropShortLabel);
    constant!(OfxPropTime);
    constant!(OfxPropType);
    constant!(OfxPropVersion);
    constant!(OfxPropVersionLabel);
}

pub mod param {
    constant!(OfxParamCoordinatesCanonical);
    constant!(OfxParamCoordinatesNormalised);
    constant!(OfxParamDoubleTypeAbsoluteTime);
    constant!(OfxParamDoubleTypeAngle);
    constant!(OfxParamDoubleTypeNormalisedX);
    constant!(OfxParamDoubleTypeNormalisedXAbsolute);
    constant!(OfxParamDoubleTypeNormalisedXY);
    constant!(OfxParamDoubleTypeNormalisedXYAbsolute);
    constant!(OfxParamDoubleTypeNormalisedY);
    constant!(OfxParamDoubleTypeNormalisedYAbsolute);
    constant!(OfxParamDoubleTypePlain);
    constant!(OfxParamDoubleTypeScale);
    constant!(OfxParamDoubleTypeTime);
    constant!(OfxParamDoubleTypeX);
    constant!(OfxParamDoubleTypeXAbsolute);
    constant!(OfxParamDoubleTypeXY);
    constant!(OfxParamDoubleTypeXYAbsolute);
    constant!(OfxParamDoubleTypeY);
    constant!(OfxParamDoubleTypeYAbsolute);
    constant!(OfxParamInvalidateAll);
    constant!(OfxParamInvalidateValueChange);
    constant!(OfxParamInvalidateValueChangeToEnd);
    constant!(OfxParamPageSkipColumn);
    constant!(OfxParamPageSkipRow);
    constant!(OfxParamPropAnimates);
    constant!(OfxParamPropCacheInvalidation);
    constant!(OfxParamPropCanUndo);
    constant!(OfxParamPropChoiceEnum);
    constant!(OfxParamPropChoiceOption);
    constant!(OfxParamPropCustomInterpCallbackV1);
    constant!(OfxParamPropCustomValue);
    constant!(OfxParamPropDataPtr);
    constant!(OfxParamPropDefault);
    constant!(OfxParamPropDefaultCoordinateSystem);
    constant!(OfxParamPropDigits);
    constant!(OfxParamPropDimensionLabel);
    constant!(OfxParamPropDisplayMax);
    constant!(OfxParamPropDisplayMin);
    constant!(OfxParamPropDoubleType);
    constant!(OfxParamPropEnabled);
    constant!(OfxParamPropEvaluateOnChange);
    constant!(OfxParamPropGroupOpen);
    constant!(OfxParamPropHasHostOverlayHandle);
    constant!(OfxParamPropHint);
    constant!(OfxParamPropIncrement);
    constant!(OfxParamPropInteractMinimumSize);
    constant!(OfxParamPropInteractPreferedSize);
    constant!(OfxParamPropInteractSize);
    constant!(OfxParamPropInteractSizeAspect);
    constant!(OfxParamPropInteractV1);
    constant!(OfxParamPropInterpolationAmount);
    constant!(OfxParamPropInterpolationTime);
    constant!(OfxParamPropIsAnimating);
    constant!(OfxParamPropIsAutoKeying);
    constant!(OfxParamPropMax);
    constant!(OfxParamPropMin);
    constant!(OfxParamPropPageChild);
    constant!(OfxParamPropParametricDimension);
    constant!(OfxParamPropParametricInteractBackground);
    constant!(OfxParamPropParametricRange);
    constant!(OfxParamPropParametricUIColour);
    constant!(OfxParamPropParent);
    constant!(OfxParamPropPersistant);
    constant!(OfxParamPropPluginMayWrite);
    constant!(OfxParamPropScriptName);
    constant!(OfxParamPropSecret);
    constant!(OfxParamPropShowTimeMarker);
    constant!(OfxParamPropStringFilePathExists);
    constant!(OfxParamPropStringMode);
    constant!(OfxParamPropType);
    constant!(OfxParamPropUseHostOverlayHandle);
    constant!(OfxParamStringIsDirectoryPath);
    constant!(OfxParamStringIsFilePath);
    constant!(OfxParamStringIsLabel);
    constant!(OfxParamStringIsMultiLine);
    constant!(OfxParamStringIsRichTextFormat);
    constant!(OfxParamStringIsSingleLine);
    constant!(OfxParamTypeBoolean);
    constant!(OfxParamTypeChoice);
    constant!(OfxParamTypeCustom);
    constant!(OfxParamTypeDouble);
    constant!(OfxParamTypeDouble2D);
    constant!(OfxParamTypeDouble3D);
    constant!(OfxParamTypeGroup);
    constant!(OfxParamTypeInteger);
    constant!(OfxParamTypeInteger2D);
    constant!(OfxParamTypeInteger3D);
    constant!(OfxParamTypePage);
    constant!(OfxParamTypeParametric);
    constant!(OfxParamTypePushButton);
    constant!(OfxParamTypeRGB);
    constant!(OfxParamTypeRGBA);
    constant!(OfxParamTypeStrChoice);
    constant!(OfxParamTypeString);
}

pub mod interact {
    pub const OfxActionDescribeInteract: &str = super::actions::OfxActionDescribe;
    pub const OfxActionCreateInstanceInteract: &str =
        super::actions::OfxActionCreateInstance;
    pub const OfxActionDestroyInstanceInteract: &str =
        super::actions::OfxActionDestroyInstance;
    constant!(OfxInteractActionDraw);
    constant!(OfxInteractActionGainFocus);
    constant!(OfxInteractActionKeyDown);
    constant!(OfxInteractActionKeyRepeat);
    constant!(OfxInteractActionKeyUp);
    constant!(OfxInteractActionLoseFocus);
    constant!(OfxInteractActionPenDown);
    constant!(OfxInteractActionPenMotion);
    constant!(OfxInteractActionPenUp);
    constant!(OfxInteractPropBackgroundColour);
    constant!(OfxInteractPropBitDepth);
    constant!(OfxInteractPropHasAlpha);
    constant!(OfxInteractPropPenPosition);
    constant!(OfxInteractPropPenPressure);
    constant!(OfxInteractPropPenViewportPosition);
    constant!(OfxInteractPropPixelScale);
    constant!(OfxInteractPropSlaveToParam);
    constant!(OfxInteractPropSuggestedColour);
    constant!(OfxInteractPropViewportSize);
}

pub mod message {
    constant!(OfxMessageError);
    constant!(OfxMessageFatal);
    constant!(OfxMessageLog);
    constant!(OfxMessageMessage);
    constant!(OfxMessageQuestion);
    constant!(OfxMessageWarning);
}

pub mod misc {
    constant!(OfxBitDepthByte);
    constant!(OfxBitDepthFloat);
    constant!(OfxBitDepthHalf);
    constant!(OfxBitDepthNone);
    constant!(OfxBitDepthShort);
    constant!(OfxChangePluginEdited);
    constant!(OfxChangeTime);
    constant!(OfxChangeUserEdited);
    constant!(OfxOpenGLPropPixelDepth);
    constant!(OfxPluginPropFilePath);
    constant!(OfxPluginPropParamPageOrder);
    constant!(OfxTypeClip);
    constant!(OfxTypeImage);
    constant!(OfxTypeImageEffect);
    constant!(OfxTypeImageEffectHost);
    constant!(OfxTypeImageEffectInstance);
    constant!(OfxTypeParameter);
    constant!(OfxTypeParameterInstance);
}
