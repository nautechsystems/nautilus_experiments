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

import pickle

from experiments.data.objects import TradeTick, TradeId

def test_pickling_tradeid():
    data = TradeId("Hello world")
    
    pickled = pickle.dumps(data)
    unpickled = pickle.loads(pickled)

    assert data == unpickled

def test_pickling_trade():
    data = TradeTick(TradeId("Hello world"), 0, 0)
    
    pickled = pickle.dumps(data)
    unpickled = pickle.loads(pickled)

    assert data == unpickled
