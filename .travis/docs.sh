#!/bin/bash

set -o errexit -o nounset

if [ "$TRAVIS_PULL_REQUEST" != "false" ]
then
    echo "This commit was part of a pull request not a merge to master. Not publishing docs"
    exit 0
fi

if [ "$TRAVIS_BRANCH" != "master" ]
then
    echo "This commit was made against the $TRAVIS_BRANCH and not the master! Not publishing docs"
    exit 0
fi

rev=$(git rev-parse --short HEAD)

cd target/doc

echo '<meta http-equiv="refresh" content="0; url=flaken/index.html">' > index.html

git init
git config user.name "Tom Burdick"
git config user.email "thomas.burdick@gmail.com"

git remote add upstream "https://$GH_TOKEN@github.com/bfrog/flaken.git"
git fetch upstream
git reset upstream/gh-pages

git add -A .
git commit -m "rebuild pages at ${rev}"
git push -q upstream HEAD:gh-pages
