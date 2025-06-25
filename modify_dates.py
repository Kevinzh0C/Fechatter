#!/usr/bin/env python3

import sys
import datetime

# 要跳过的提交（Meilisearch and NATS configuration）
SKIP_COMMIT = "67315cadb346c5d12981f6c9401ff72a98742a1b"

def modify_commit_date(commit_info):
    # 解析输入
    lines = commit_info.strip().split('\n')
    commit_hash = lines[0]
    author_date = lines[1]
    committer_date = lines[2]
    
    # 如果是要跳过的提交，返回原始日期
    if commit_hash == SKIP_COMMIT:
        return f"{author_date}\n{committer_date}"
    
    # 解析日期并添加4个月
    try:
        # Git 日期格式：timestamp timezone
        author_parts = author_date.split()
        committer_parts = committer_date.split()
        
        author_timestamp = int(author_parts[0])
        author_timezone = author_parts[1]
        
        committer_timestamp = int(committer_parts[0])
        committer_timezone = committer_parts[1]
        
        # 转换为 datetime
        author_dt = datetime.datetime.fromtimestamp(author_timestamp)
        committer_dt = datetime.datetime.fromtimestamp(committer_timestamp)
        
        # 添加4个月（120天）
        author_new = author_dt + datetime.timedelta(days=120)
        committer_new = committer_dt + datetime.timedelta(days=120)
        
        # 转换回 timestamp
        author_new_timestamp = int(author_new.timestamp())
        committer_new_timestamp = int(committer_new.timestamp())
        
        # 返回新的日期
        return f"{author_new_timestamp} {author_timezone}\n{committer_new_timestamp} {committer_timezone}"
        
    except Exception as e:
        # 如果出错，返回原始日期
        return f"{author_date}\n{committer_date}"

if __name__ == "__main__":
    commit_info = sys.stdin.read()
    result = modify_commit_date(commit_info)
    print(result, end='')

