#!/usr/bin/env bash
{
git remote update
git fetch
branch_name=$(date +%s%3N)
cd Tetris-Community
git checkout -b $branch_name
cat > tetriscommunity.md
git add tetriscommunity.md
git commit -m "$1" -m "$2"
git push --set-upstream origin $branch_name
git checkout main
git branch --delete $branch_name
} 2>&1