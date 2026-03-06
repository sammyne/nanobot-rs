#!/bin/bash

cd `dirname ${BASH_SOURCE[0]}`

TAVILY_REV=f63aeef

wget https://github.com/tavily-ai/skills/archive/$TAVILY_REV.tar.gz -O /tmp/tavily-skills.tgz

rm -rf /tmp/tavily-ai-skills
mkdir -p /tmp/tavily-ai-skills

tar -xzf /tmp/tavily-skills.tgz --strip-components=1 -C /tmp/tavily-ai-skills

mv /tmp/tavily-ai-skills/skills/tavily/search tavily-search

sed -i '/^description:/,/^---$/s/^---$/metadata:\
  nanobot:\
    requires:\
      env: ["TAVILY_API_KEY"]\
&/' /github.com/sammyne/nanobot-rs/crates/skills/builtin/tavily-search/SKILL.md
