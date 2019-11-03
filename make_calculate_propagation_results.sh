#!/bin/bash
# SPDX-License-Identifier: LGPL-2.1-or-later
# See Notices.txt for copyright information
set -e

names=(first second third)

for order in '0 1 2' '0 2 1' '1 0 2' '1 2 0' '2 0 1' '2 1 0'; do
    for i in $order; do
        echo -n "${names[i]^}"
    done
    echo " => {"
    echo -n "    "
    for i in $order; do
        echo "if ${names[i]}_class.is_nan() {"
        echo "        ${names[i]^}"
        echo -n "    } else "
    done
    echo "{"
    echo "        Canonical"
    echo "    }"
    echo "}"
    for i in $order; do
        echo -n "${names[i]^}"
    done
    echo "PreferringSNaN => {"
    echo -n "    "
    for i in $order; do
        echo "if ${names[i]}_class.is_signaling_nan() {"
        echo "        ${names[i]^}"
        echo -n "    } else "
    done
    for i in $order; do
        echo "if ${names[i]}_class.is_nan() {"
        echo "        ${names[i]^}"
        echo -n "    } else "
    done
    echo "{"
    echo "        Canonical"
    echo "    }"
    echo "}"
done
