#!/bin/bash
cp -r /Users/matkat/Sofware/protos/ ./protos
docker build -t crawler . --progress=plain