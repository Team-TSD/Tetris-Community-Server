#!/usr/bin/env bash
cd Tetris-Community
git fetch --all
git branch backup-main
git reset --hard origin/main