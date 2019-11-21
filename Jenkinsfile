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
    parallel(test_cli_macos: {
        node('osx') {
            checkout(scm)
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
    ffi_ios_aarch64: {
        node('osx') {
            checkout(scm)
            ["prod", "dev"].each({
                runReleaseBuild("safe-ffi", "${it}", "aarch64-apple-ios")
                packageBuildArtifacts("safe-ffi", "${it}", "aarch64-apple-ios")
                uploadBuildArtifacts()
            })
        }
    },
    ffi_ios_x86_64: {
        node('osx') {
            checkout(scm)
            ["prod", "dev"].each({
                runReleaseBuild("safe-ffi", "${it}", "x86_64-apple-ios")
                packageBuildArtifacts("safe-ffi", "${it}", "x86_64-apple-ios")
                uploadBuildArtifacts()
            })
        }
    },
    ffi_macos: {
        node('osx') {
            checkout(scm)
            ["prod", "dev"].each({
                runReleaseBuild("safe-ffi", "${it}", "x86_64-apple-darwin")
                stripArtifacts()
                packageBuildArtifacts("safe-ffi", "${it}", "x86_64-apple-darwin")
                uploadBuildArtifacts()  
            })
        }
    },
    cli_macos: {
        node('osx') {
            checkout(scm)
            ["prod", "dev"].each({
                runReleaseBuild("safe-cli", "${it}", "x86_64-apple-darwin")
                stripArtifacts()
                packageBuildArtifacts("safe-cli", "${it}", "x86_64-apple-darwin")
                uploadBuildArtifacts()
            })
        }
    })
}

stage("build universal iOS lib") {
    node("osx") {
        checkout(scm)
        def branch = env.CHANGE_ID?.trim() ?: env.BRANCH_NAME
        withEnv(["SAFE_CLI_BRANCH=${branch}",
                 "SAFE_CLI_BUILD_NUMBER=${env.BUILD_NUMBER}"]) {
            sh("make retrieve-ios-build-artifacts")
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
            packageArtifactsForDeploy(false)
            uploadDeployArtifacts("dev")
            uploadDeployArtifacts("prod")
        } else {
            echo("${env.BRANCH_NAME} does not match the deployment branch. Nothing to do.")
        }
    }
}

def runReleaseBuild(component, type, target) {
    // Running a dev build as a clean build is very slow if we're trying to do
    // both the prod and dev builds as part of the same job.
    def cleanBuild = env.BRANCH_NAME == "${params.CLEAN_BUILD_BRANCH}" && type != "dev"
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

def packageArtifactsForDeploy(isVersionCommit) {
    if (isVersionCommit) {
        sh("make package-version-artifacts-for-deploy")
    } else {
        sh("make package-commit_hash-artifacts-for-deploy")
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
