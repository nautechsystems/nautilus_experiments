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

import gc

from experiments.data.objects import QuoteTick, InstrumentId, Symbol


def test_string_value():
    symbol = Symbol("hello world")
    symbol.debug()
    instrument = InstrumentId(symbol)
    instrument.debug()
    data = [QuoteTick(instrument) for _ in range(10000000)]
    data[-1].debug()


def test_from_string_value():
    instrument = InstrumentId.from_string("hello world")
    instrument.debug()
    data = [QuoteTick(instrument) for _ in range(10000000)]
    data[-1].debug()


def test_large_allocation():
    for i in range(5):
        instrument = InstrumentId.from_string("hello world")
        instrument.debug()
        data = [QuoteTick(instrument) for _ in range(20000000)]
        data[-1].debug()
        
    gc.collect()
