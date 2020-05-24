#!/usr/bin/env ts-node-script

import { getPlumberEnvVars } from '../lib/plumberEnv'
import { fetch as gitFetch, getCommonAncestor, getCommitHashesList, checkIfSigned } from '../lib/git'
import { execWithStringReturn } from '../lib/exec'
import { getPackageJsonVersionFromCommit, getCargoTomlVersionFromCommit } from '../lib/version'

async function checkCommitMessages(commonAncestor: string): Promise<void> {
  console.log('Validating commit messages')
  await execWithStringReturn(`npm run commitlint -- --from ${commonAncestor}`)
}

async function checkCommitSignatures(commitsList: string[]): Promise<void> {
  console.log('Checking that commits are signed (but not verifying signatures)')
  for (const commitHash of commitsList) {
    if (!(await checkIfSigned(commitHash))) {
      console.error(`ERROR: Commit ${commitHash} is not signed`)
      process.exit(1)
    }
  }
}

async function checkVersions(currentGitRef: string): Promise<void> {
  const packageJsonVersion = (await getPackageJsonVersionFromCommit(currentGitRef)).join('.')
  const cargoTomlVersion = (await getCargoTomlVersionFromCommit(currentGitRef)).join('.')

  if (packageJsonVersion !== cargoTomlVersion) {
    console.error(
      `ERROR: package.json version (${packageJsonVersion}) and Cargo.tom version (${cargoTomlVersion}) are different`
    )
    process.exit(1)
  }
}

async function run(): Promise<void> {
  const env = await getPlumberEnvVars()

  // We do this so we can "compare" branches
  await gitFetch()

  const commonAncestor = await getCommonAncestor(env.PLUMBER_GIT_MASTER_REF, env.CI_COMMIT_SHA)
  const commitsList = await getCommitHashesList(commonAncestor, env.CI_COMMIT_SHA)

  await checkCommitMessages(commonAncestor)
  await checkCommitSignatures(commitsList)
  await checkVersions(env.CI_COMMIT_SHA)
}

run()
  .then(() => {
    console.log('Finished git checks')
  })
  .catch(reason => {
    console.log(reason)
    process.exit(1)
  })
