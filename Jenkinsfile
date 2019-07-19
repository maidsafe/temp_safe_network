properties([
    parameters([
        string(name: 'ARTIFACTS_BUCKET', defaultValue: 'safe-jenkins-build-artifacts'),
        string(name: 'DEPLOY_BUCKET', defaultValue: 'safe-cli')
    ])
])

stage('build & test') {
    parallel linux: {
        node('docker') {
            checkout(scm)
            runTests()
            packageBuildArtifacts('linux')
            uploadBuildArtifacts()
        }
    },
    windows: {
        node('windows') {
            checkout(scm)
            runTests()
            packageBuildArtifacts('windows')
            uploadBuildArtifacts()
        }
    },
    macos: {
        node('osx') {
            checkout(scm)
            runTests()
            packageBuildArtifacts('macos')
            uploadBuildArtifacts()
        }
    }
}

stage('deploy') {
    node('docker') {
        if (env.BRANCH_NAME == "master") {
            checkout(scm)
            sh("git fetch --tags --force")
            retrieveBuildArtifacts()
            if (versionChangeCommit()) {
                version = sh(
                    returnStdout: true,
                    script: "grep '^version' < Cargo.toml | head -n 1 | awk '{ print \$3 }' | sed 's/\"//g'").trim()
                packageArtifactsForDeploy(true)
                createTag(version)
                createGithubRelease(version)
            } else {
                packageArtifactsForDeploy(false)
                uploadDeployArtifacts()
            }
        } else {
            echo("${env.BRANCH_NAME} does not match the deployment branch. Nothing to do.")
        }
    }
}

def runTests() {
    command_prefix = "RANDOM_PORT_NUMBER=\$(( \$RANDOM % 100 + 41800 )) "
    try {
        test_command = command_prefix + "make test"
        sh(test_command)
    } finally {
        clean_command = command_prefix + "make clean"
        sh(clean_command)
    }
}

def versionChangeCommit() {
    shortCommitHash = sh(
        returnStdout: true,
        script: "git log -n 1 --pretty=format:'%h'").trim()
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
    withCredentials([usernamePassword(
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
    withCredentials([usernamePassword(
        credentialsId: "github_maidsafe_token_credentials",
        usernameVariable: "GITHUB_USER",
        passwordVariable: "GITHUB_TOKEN")]) {
        sh("make deploy-github-release")
    }
}

def retrieveBuildArtifacts() {
    command = ""
    if (env.CHANGE_ID?.trim()) {
        command += "SAFE_CLI_BRANCH=${env.CHANGE_ID} "
    } else {
        command += "SAFE_CLI_BRANCH=${env.BRANCH_NAME} "
    }
    command += "SAFE_CLI_BUILD_NUMBER=${env.BUILD_NUMBER} "
    command += "make retrieve-all-build-artifacts"
    sh(command)
}

def packageBuildArtifacts(os) {
    command = ""
    if (env.CHANGE_ID?.trim()) {
        command += "SAFE_CLI_BRANCH=${env.CHANGE_ID} "
    } else {
        command += "SAFE_CLI_BRANCH=${env.BRANCH_NAME} "
    }
    command += "SAFE_CLI_BUILD_NUMBER=${env.BUILD_NUMBER} "
    command += "SAFE_CLI_BUILD_OS=${os} "
    command += "make package-build-artifacts"
    sh(command)
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

def uploadDeployArtifacts() {
    withAWS(credentials: 'aws_jenkins_deploy_artifacts_user', region: 'eu-west-2') {
        def artifacts = sh(returnStdout: true, script: 'ls -1 deploy').trim().split("\\r?\\n")
        for (artifact in artifacts) {
            s3Upload(
                bucket: "${params.DEPLOY_BUCKET}",
                file: artifact,
                workingDir: "${env.WORKSPACE}/deploy",
                acl: 'PublicRead')
        }
    }
}
