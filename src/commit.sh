#!/usr/bin/env bash
{
git remote update
git fetch
branch_name=$(date +%s%3N)
cd Tetris-Community
git checkout -b $branch_name
cat > tetriscommunity.md
if [ $3 -eq 1 ]
then
    echo "$1" >> contributors.txt
fi
git add tetriscommunity.md
git add contributors.txt
git commit -m "autocommit from $1" -m "$2"
git push --set-upstream origin $branch_name
git checkout main
git branch --delete $branch_name
} 2>&1