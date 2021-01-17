#!/bin/sh

set -e

# Get rustfmt goin
rustup component add rustfmt

### Setup python linker flags ##################################################

python -c """
import sysconfig
cfg = sorted(sysconfig.get_config_vars().items())
print('\n'.join(['{}={}'.format(*x) for x in cfg]))
"""

export PYTHON_LIB=$(python -c "import sysconfig as s; print(s.get_config_var('LIBDIR'))")

# find $PYTHON_LIB
export LIBRARY_PATH="$LIBRARY_PATH:$PYTHON_LIB"

# delete any possible empty components
# https://github.com/google/pulldown-cmark/issues/122#issuecomment-364948741
LIBRARY_PATH=$(echo $LIBRARY_PATH | sed -E -e 's/^:*//' -e 's/:*$//' -e 's/:+/:/g')

export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB:$HOME/rust/lib"