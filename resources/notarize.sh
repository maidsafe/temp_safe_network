#!/usr/bin/env bash

build_app=$1
if [[ -z "$build_app" ]]; then
    echo "You must supply an app to notarize."
    exit 1
fi

build_app_location="./target/release/$build_app"


if [[ -z "$APPLE_SIGN_ID" ]]; then
    echo "You must supply an apple sign id for signing"
    exit 1
fi
if [[ -z "$APPLE_ID" ]]; then
    echo "You must supply an apple id"
    exit 1
fi

if [[ -z "$APPLE_ID_PASSWORD" ]]; then
    echo "You must supply an apple password."
    exit 1
fi

if [[ -z "$CSC_LINK" ]]; then
    echo "You must supply a base64 encoded certificate."
    exit 1
fi

if [[ -z "$CSC_KEY_PASSWORD" ]]; then
    echo "You must supply certificate password."
    exit 1
fi

# security find-identity -v

echo $CSC_LINK | base64 --decode > certificate.p12

echo "certificate prepared..."
# CSC_KEY_PASSWORD used here as just any pass will do
security create-keychain -p $CSC_KEY_PASSWORD build.keychain
security default-keychain -s build.keychain
security unlock-keychain -p $CSC_KEY_PASSWORD build.keychain

# import cert and set as trusted so no prompt is needed to accept.
security import certificate.p12 -k build.keychain -P $CSC_KEY_PASSWORD -T /usr/bin/codesign
# sudo security add-trusted-cert -d -r trustRoot -k build.keychain certificate.p12
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k $CSC_KEY_PASSWORD build.keychain
# security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k $KEYCHAIN_PASSWORD circle.keychain

security find-identity -v



echo "Attempting to sign: $build_app_location"
echo "Notarization: signing."
# first lets sign the bin
codesign --force --deep --timestamp --verbose --options runtime --sign $APPLE_SIGN_ID  $build_app_location
echo "Singed successfully."

echo "Notarization: zipping."
# then zip, which is needed for notarize
ditto -c -k --rsrc $build_app_location "$build_app_location.zip"

ls target/release
#strip dots and underscores
bundleid=${build_app%.*}
bundleid=${bundleid%_*}

echo "Notarization: uploading $build_app_location.zip"
# trigger the notarize
xcrun altool --notarize-app -f "$build_app_location.zip" --primary-bundle-id "com.maidsafe.$bundleid" -u "$APPLE_ID" -p "$APPLE_ID_PASSWORD" &> tmp

echo "Upload response: "
cat tmp
echo 'Notarization: waiting.'
# and wait for complete
uuid=`cat tmp | grep -Eo '\w{8}-(\w{4}-){3}\w{12}$'`


if [[ -z "$uuid" ]]; then
    echo "There was an issue getting the UUID from response"
    cat tmp
    exit 1
fi

echo "UUID received: $uuid"

TOTAL_WAIT_TIME=0
TIMEOUT_SECONDS=1200 # 20mins * 60

while [ "$TOTAL_WAIT_TIME" -lt "$TIMEOUT_SECONDS" ]; do
    sleep 60
    TOTAL_WAIT_TIME=$(($TOTAL_WAIT_TIME + 60))

    echo "Checking notarization status"

    xcrun altool --notarization-info "$uuid" --username "$APPLE_ID" --password "$APPLE_ID_PASSWORD" &> tmp
    r=`cat tmp`
    echo "$r"
    t=`echo "$r" | grep "success"`
    f=`echo "$r" | grep "invalid"`
    if [[ "$t" != "" ]]; then
        echo "Notarization successful!"
        xcrun stapler staple "$build_app_location"
        xcrun stapler staple "$build_app_location.zip"
        echo "Notarization stapled to bins successfully"
        exit 0;
        break
    fi
    if [[ "$f" != "" ]]; then
        echo "$r"
        exit 1
    fi
    echo "Waiting on notariation... sleep 2m then check again..."

done
