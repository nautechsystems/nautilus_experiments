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

from experiments.data.objects import Symbol


class TestQuoteTick:
    def test_make_quote_tick(self):
        symbol=Symbol("AUD/USD")
        print(f"python side symbol: {symbol._mem}")
        new_symbol = Symbol.from_raw_py(symbol)
        print(f"python side symbol after new symbol: {symbol._mem}")
        del symbol
