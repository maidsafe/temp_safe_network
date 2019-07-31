properties([
    parameters([
        string(name: "ARTIFACTS_BUCKET", defaultValue: "safe-jenkins-build-artifacts"),
        string(name: "DEPLOY_BUCKET", defaultValue: "safe-vault")
    ])
])

stage("build & test") {
    parallel linux: {
        node("safe_vault") {
            checkout(scm)
            sh("make test")
            packageBuildArtifacts("linux")
            uploadBuildArtifacts()
        }
    },
    windows: {
        node("windows") {
            checkout(scm)
            sh("make test")
            packageBuildArtifacts("windows")
            uploadBuildArtifacts()
        }
    },
    macos: {
        node("osx") {
            checkout(scm)
            sh("make test")
            packageBuildArtifacts("macos")
            uploadBuildArtifacts()
        }
    },
    clippy: {
        node("safe_vault") {
            checkout(scm)
            sh("make clippy")
        }
    }
}

def packageBuildArtifacts(os) {
    branch = env.CHANGE_ID?.trim() ?: env.BRANCH_NAME
    withEnv(["SAFE_VAULT_BRANCH=${branch}",
             "SAFE_VAULT_BUILD_NUMBER=${env.BUILD_NUMBER}",
             "SAFE_VAULT_BUILD_OS=${os}"]) {
        sh("make package-build-artifacts")
    }
}

def uploadBuildArtifacts() {
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
