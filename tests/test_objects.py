# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2022 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------

import sys
import gc
import tracemalloc

from experiments.data.objects import UUID4


def test_large_unique_allocation():
    for i in range(1):
        data = [UUID4("550e8400-e29b-41d4-a716-446655" + str(n).zfill(6)) for n in range(4000)]
        print(data[-1])
        
    gc.collect()


def test_large_allocation():
    for i in range(5):
        data = [UUID4("550e8400-e29b-41d4-a716-446655440000") for n in range(400000)]
        print(data[-1])
        
    gc.collect()


def test_large_printing():
    for _ in range(5):
        for n in range(10000):
            a = UUID4("550e8400-e29b-41d4-a716-44665544" + str(n % 10000).zfill(4))
            print(a)
        gc.collect()
    gc.collect()


def test_large_pickling():
    import pickle
    uuid = UUID4("550e8400-e29b-41d4-a716-446655440000")
    uuid_obj = pickle.dumps(uuid)
    for _ in range(5):
        for _ in range(1000):
            print(pickle.loads(uuid_obj))
        gc.collect()


if __name__ == "__main__":
    tracemalloc.start()
    snap1 = tracemalloc.take_snapshot()

    old_stdout = sys.stdout
    f = open('/dev/null', 'w')
    sys.stdout = f
    test_large_printing()
    gc.collect()
    f.close()
    sys.stdout = old_stdout

    snap2 = tracemalloc.take_snapshot()

    top_stats = snap2.compare_to(snap1, 'lineno')

    print("[ Top 10 differences ]")
    for stat in top_stats[:10]:
        print(stat)

    top_stats = snap2.statistics('traceback')

    # pick the biggest memory block
    stat = top_stats[0]
    print("%s memory blocks: %.1f KiB" % (stat.count, stat.size / 1024))
    for line in stat.traceback.format():
        print(line)
