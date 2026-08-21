#!/bin/zsh
# rd.sh <Class/method@addr> [from] [to]  -- print a decompiled function with the String boilerplate stripped
f=~/ghidra-projects/decomp/funcs/$1.c
from=${2:-1}; to=${3:-400}
grep -v -E 'String::close|local_[0-9a-f]+ = |\.hash_value|module_id|\.flags =|\.offset|curr_len|const_len|hash_value_insensitive|ExceptionList|puStack|^\s*$' "$f" | sed -n "${from},${to}p"
