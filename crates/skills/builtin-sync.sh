#!/bin/bash

# 这个脚本用于同步部分内置技能。

cd `dirname ${BASH_SOURCE[0]}`/builtin

## Tavily 搜索
TAVILY_REV=f63aeef

wget https://github.com/tavily-ai/skills/archive/$TAVILY_REV.tar.gz -O /tmp/tavily-skills.tgz

rm -rf /tmp/tavily-ai-skills
mkdir -p /tmp/tavily-ai-skills

tar -xzf /tmp/tavily-skills.tgz --strip-components=1 -C /tmp/tavily-ai-skills

rm -rf tavily-search
mv /tmp/tavily-ai-skills/skills/tavily/search tavily-search

sed -i '/^description:/,/^---$/s/^---$/metadata:\
  nanobot:\
    requires:\
      bins: ["jq"]\
      env: ["TAVILY_API_KEY"]\
&/' /github.com/sammyne/nanobot-rs/crates/skills/builtin/tavily-search/SKILL.md

# Replace the description with the desired content
sed -i 's/^description:.*/description: "Search the web for real-time information using Tavily'\''s LLM-optimized API. Use this skill when you need: weather forecasts and conditions, latest news and current events, stock prices and financial data, sports scores and schedules, trending topics and breaking news, or any time-sensitive information beyond your knowledge cutoff. Returns relevant results with content snippets, scores, and metadata."/' tavily-search/SKILL.md
