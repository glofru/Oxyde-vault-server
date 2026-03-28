#!/bin/bash
git clone https://"$USERNAME":"$PERSONAL_ACCESS_TOKEN"@github.com/"$USERNAME"/"$REPOSITORY_NAME".git
/usr/local/bin/Oxyde-vault-server