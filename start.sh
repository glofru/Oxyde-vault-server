#!/bin/bash
git clone https://"$USERNAME":"$PERSONAL_ACCESS_TOKEN"@github.com/"$USERNAME"/"$REPOSITORY_NAME".git --depth 1
/usr/local/bin/Oxyde-vault-server