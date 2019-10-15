properties([
    parameters([
        string(name: 'ARTIFACTS_BUCKET', defaultValue: 'safe-jenkins-build-artifacts'),
        string(name: 'CACHE_BRANCH', defaultValue: 'master'),
        string(name: 'DEPLOY_BRANCH', defaultValue: 'master'),
        string(name: 'PUBLISH_BRANCH', defaultValue: 'master'),
        string(name: 'DEPLOY_BUCKET', defaultValue: 'safe-cli'),
        string(name: 'CLEAN_BUILD_BRANCH', defaultValue: 'master')
    ])
])

stage('build & test') {
    parallel test_linux: {
        node('safe_cli') {
            checkout(scm)
            runTests("cli")
            packageBuildArtifacts("safe-cli", "dev", "x86_64-unknown-linux-gnu")
            uploadBuildArtifacts()
        }
    },
    test_cli_windows: {
        node('windows') {
            checkout(scm)
            retrieveCache('windows')
            runTests("cli")
            packageBuildArtifacts("safe-cli", "dev", "x86_64-pc-windows-gnu")
            uploadBuildArtifacts()
        }
    },
    test_cli_macos: {
        node('osx') {
            checkout(scm)
            retrieveCache('macos')
            runTests("cli")
            packageBuildArtifacts("safe-cli", "dev", "x86_64-apple-darwin")
            uploadBuildArtifacts()
        }
    },
    test_api_macos: {
        node('osx') {
            checkout(scm)
            runTests("api")
        }
    },
    test_api_windows: {
        node('windows') {
            checkout(scm)
            runTests("api")
        }
    },
    test_api_linux: {
        node('safe_cli') {
            checkout(scm)
            runTests("api")
        }
    },
    clippy: {
        node('safe_cli') {
            checkout(scm)
            sh("make clippy")
        }
    },
    release_cli_linux: {
        node('safe_cli') {
            checkout(scm)
            runReleaseBuild("safe-cli", "non-dev", "x86_64-unknown-linux-gnu")
            stripArtifacts()
            packageBuildArtifacts("safe-cli", "non-dev", "x86_64-unknown-linux-gnu")
            uploadBuildArtifacts()
        }
    },
    release_cli_windows: {
        node('windows') {
            checkout(scm)
            runReleaseBuild("safe-cli", "non-dev", "x86_64-pc-windows-gnu")
            stripArtifacts()
            packageBuildArtifacts("safe-cli", "non-dev", "x86_64-pc-windows-gnu")
            uploadBuildArtifacts()
        }
    },
    release_cli_macos: {
        node('osx') {
            checkout(scm)
            runReleaseBuild("safe-cli", "non-dev", "x86_64-apple-darwin")
            stripArtifacts()
            packageBuildArtifacts("safe-cli", "non-dev", "x86_64-apple-darwin")
            uploadBuildArtifacts()
        }
    },
    release_ffi_macos: {
        node('osx') {
            checkout(scm)
            runReleaseBuild("safe-ffi", "non-dev", "x86_64-apple-darwin")
            stripArtifacts()
            packageBuildArtifacts("safe-ffi", "non-dev", "x86_64-apple-darwin")
            uploadBuildArtifacts()
        }
    },
    release_ffi_windows: {
        node('windows') {
            checkout(scm)
            runReleaseBuild("safe-ffi", "non-dev", "x86_64-pc-windows-gnu")
            packageBuildArtifacts("safe-ffi", "non-dev", "x86_64-pc-windows-gnu")
            uploadBuildArtifacts()
        }
    },
    release_ffi_linux: {
        node('safe_cli') {
            checkout(scm)
            runReleaseBuild("safe-ffi", "non-dev", "x86_64-unknown-linux-gnu")
            stripArtifacts()
            packageBuildArtifacts("safe-ffi", "non-dev", "x86_64-unknown-linux-gnu")
            uploadBuildArtifacts()
        }
    },
    release_ffi_android_x86_64: {
        node('safe_cli') {
            checkout(scm)
            runReleaseBuild("safe-ffi", "non-dev", "x86_64-linux-android")
            packageBuildArtifacts("safe-ffi", "non-dev", "x86_64-linux-android")
            uploadBuildArtifacts()
        }
    },
    release_ffi_android_armv7: {
        node('safe_cli') {
            checkout(scm)
            runReleaseBuild("safe-ffi", "non-dev", "armv7-linux-androideabi")
            packageBuildArtifacts("safe-ffi", "non-dev", "armv7-linux-androideabi")
            uploadBuildArtifacts()
        }
    },
    release_ffi_ios_aarch64: {
        node("osx") {
            checkout(scm)
            runReleaseBuild("safe-ffi", "non-dev", "aarch64-apple-ios")
            packageBuildArtifacts("safe-ffi", "non-dev", "aarch64-apple-ios")
            uploadBuildArtifacts()
        }
    },
    release_ffi_ios_x86_64: {
        node("osx") {
            checkout(scm)
            runReleaseBuild("safe-ffi", "non-dev", "x86_64-apple-ios")
            packageBuildArtifacts("safe-ffi", "non-dev", "x86_64-apple-ios")
            uploadBuildArtifacts()
        }
    },
    dev_ffi_macos: {
        node('osx') {
            checkout(scm)
            runReleaseBuild("safe-ffi", "dev", "x86_64-apple-darwin")
            stripArtifacts()
            packageBuildArtifacts("safe-ffi", "dev", "x86_64-apple-darwin")
            uploadBuildArtifacts()
        }
    },
    dev_ffi_windows: {
        node('windows') {
            checkout(scm)
            runReleaseBuild("safe-ffi", "dev", "x86_64-pc-windows-gnu")
            packageBuildArtifacts("safe-ffi", "dev", "x86_64-pc-windows-gnu")
            uploadBuildArtifacts()
        }
    },
    dev_ffi_linux: {
        node('safe_cli') {
            checkout(scm)
            runReleaseBuild("safe-ffi", "dev", "x86_64-unknown-linux-gnu")
            stripArtifacts()
            packageBuildArtifacts("safe-ffi", "dev", "x86_64-unknown-linux-gnu")
            uploadBuildArtifacts()
        }
    },
    dev_ffi_android_armv7: {
        node('safe_cli') {
            checkout(scm)
            runReleaseBuild("safe-ffi", "dev", "armv7-linux-androideabi")
            packageBuildArtifacts("safe-ffi", "dev", "armv7-linux-androideabi")
            uploadBuildArtifacts()
        }
    },
    dev_ffi_android_x86_64: {
        node('safe_cli') {
            checkout(scm)
            runReleaseBuild("safe-ffi", "dev", "x86_64-linux-android")
            packageBuildArtifacts("safe-ffi", "dev", "x86_64-linux-android")
            uploadBuildArtifacts()
        }
    },
    dev_ffi_ios_aarch64: {
        node("osx") {
            checkout(scm)
            runReleaseBuild("safe-ffi", "dev", "aarch64-apple-ios")
            packageBuildArtifacts("safe-ffi", "dev", "aarch64-apple-ios")
            uploadBuildArtifacts()
        }
    },
    dev_ffi_ios_x86_64: {
        node("osx") {
            checkout(scm)
            runReleaseBuild("safe-ffi", "dev", "x86_64-apple-ios")
            packageBuildArtifacts("safe-ffi", "dev", "x86_64-apple-ios")
            uploadBuildArtifacts()
        }
    }
}

stage("build universal iOS lib") {
    node("osx") {
        checkout(scm)
        def branch = env.CHANGE_ID?.trim() ?: env.BRANCH_NAME
        withEnv(["SAFE_CLI_BRANCH=${branch}",
                 "SAFE_CLI_BUILD_NUMBER=${env.BUILD_NUMBER}"]) {
            sh("make universal-ios-lib")
            sh("make package-universal-ios-lib")
            uploadBuildArtifacts()
        }
    }
}

stage('deploy') {
    node('safe_cli') {
        if (env.BRANCH_NAME == "${params.DEPLOY_BRANCH}") {
            checkout(scm)
            sh("git fetch --tags --force")
            retrieveBuildArtifacts()
            if (isVersionChangeCommit()) {
                version = sh(
                    returnStdout: true,
                    script: "grep '^version' < safe-cli/Cargo.toml | head -n 1 | awk '{ print \$3 }' | sed 's/\"//g'").trim()
                packageArtifactsForDeploy(true)
                createTag(version)
                createGithubRelease(version)
                uploadDeployArtifacts("mock")
                uploadDeployArtifacts("real")
            } else {
                packageArtifactsForDeploy(false)
                uploadDeployArtifacts("mock")
                uploadDeployArtifacts("real")
            }
        } else {
            echo("${env.BRANCH_NAME} does not match the deployment branch. Nothing to do.")
        }
    }
    if (env.BRANCH_NAME == "master") {
        build(job: "../rust_cache_build-safe_cli", wait: false)
        build(job: "../docker_build-safe_cli_build_container", wait: false)
    }
}

stage("publishing") {
    node("safe_cli") {
        checkout(scm)
        if (shouldPublish()) {
            withCredentials(
                [string(
                    credentialsId: "crates_io_token", variable: "CRATES_IO_TOKEN")]) {
                sh("make publish-api")
            }
        } else {
            echo("Not publishing.")
            echo("Not a version change commit or the publish branch doesn't match.")
        }
    }
}

def shouldPublish() {
    return isVersionChangeCommit() && env.BRANCH_NAME == "${params.PUBLISH_BRANCH}"
}

def retrieveCache(os) {
    if (!fileExists("target")) {
        withEnv(["SAFE_CLI_BRANCH=${params.CACHE_BRANCH}",
                 "SAFE_CLI_OS=${os}"]) {
            sh("make retrieve-cache")
        }
    }
}

def runReleaseBuild(component, type, target) {
    def cleanBuild = env.BRANCH_NAME == "${params.CLEAN_BUILD_BRANCH}"
    withEnv(["SAFE_CLI_BUILD_COMPONENT=${component}",
             "SAFE_CLI_BUILD_TYPE=${type}",
             "SAFE_CLI_BUILD_CLEAN=${cleanBuild}",
             "SAFE_CLI_BUILD_TARGET=${target}"]) {
        sh("make build-component")
    }
}

def stripArtifacts() {
    sh("make strip-artifacts")
}

def runTests(component) {
    def port = new Random().nextInt() % 100 + 41800
    echo("Generated ${port} at random to be used as SAFE_AUTH_PORT")
    withEnv(["SAFE_AUTH_PORT=${port}"]) {
        try {
            sh("make test-${component}")
        } finally {
            sh("make clean")
        }
    }
}

def isVersionChangeCommit() {
    shortCommitHash = sh(
        returnStdout: true,
        script: "git log -n 1 --no-merges --pretty=format:'%h'").trim()
    message = sh(
        returnStdout: true,
        script: "git log --format=%B -n 1 ${shortCommitHash}").trim()
    return message.startsWith("Version change")
}

def packageArtifactsForDeploy(isVersionCommit) {
    if (isVersionCommit) {
        sh("make package-version-artifacts-for-deploy")
    } else {
        sh("make package-commit_hash-artifacts-for-deploy")
    }
}

def createTag(version) {
    withCredentials(
        [usernamePassword(
            credentialsId: "github_maidsafe_qa_user_credentials",
            usernameVariable: "GIT_USER",
            passwordVariable: "GIT_PASSWORD")]) {
        sh("git config --global user.name \$GIT_USER")
        sh("git config --global user.email qa@maidsafe.net")
        sh("git config credential.username \$GIT_USER")
        sh("git config credential.helper '!f() { echo password=\$GIT_PASSWORD; }; f'")
        sh("git tag -a ${version} -m 'Creating tag for ${version}'")
        sh("GIT_ASKPASS=true git push origin --tags")
    }
}

def createGithubRelease(version) {
    withCredentials(
        [usernamePassword(
            credentialsId: "github_maidsafe_token_credentials",
            usernameVariable: "GITHUB_USER",
            passwordVariable: "GITHUB_TOKEN")]) {
        sh("make deploy-github-release")
    }
}

def retrieveBuildArtifacts() {
    branch = env.CHANGE_ID?.trim() ?: env.BRANCH_NAME
    withEnv(["SAFE_CLI_BRANCH=${branch}",
             "SAFE_CLI_BUILD_NUMBER=${env.BUILD_NUMBER}"]) {
        sh("make retrieve-all-build-artifacts")
    }
}

def packageBuildArtifacts(component, type, target) {
    def branch = env.CHANGE_ID?.trim() ?: env.BRANCH_NAME
    withEnv(["SAFE_CLI_BRANCH=${branch}",
             "SAFE_CLI_BUILD_NUMBER=${env.BUILD_NUMBER}",
             "SAFE_CLI_BUILD_TYPE=${type}",
             "SAFE_CLI_BUILD_COMPONENT=${component}",
             "SAFE_CLI_BUILD_TARGET=${target}"]) {
        sh("make package-build-artifacts")
    }
}

def uploadBuildArtifacts(type='') {
    withAWS(credentials: 'aws_jenkins_build_artifacts_user', region: 'eu-west-2') {
        def artifacts = sh(returnStdout: true, script: 'ls -1 artifacts').trim().split("\\r?\\n")
        for (artifact in artifacts) {
            s3Upload(
                bucket: "${params.ARTIFACTS_BUCKET}",
                file: artifact,
                workingDir: "${env.WORKSPACE}/artifacts",
                acl: 'PublicRead')
        }
    }
}

def uploadDeployArtifacts(type) {
    withAWS(credentials: 'aws_jenkins_deploy_artifacts_user', region: 'eu-west-2') {
        def artifacts = sh(
            returnStdout: true, script: "ls -1 deploy/${type}").trim().split("\\r?\\n")
        for (artifact in artifacts) {
            s3Upload(
                bucket: "${params.DEPLOY_BUCKET}",
                file: artifact,
                workingDir: "${env.WORKSPACE}/deploy/${type}",
                acl: 'PublicRead')
        }
    }
}
