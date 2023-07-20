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

from experiments.data.objects import QuoteTick, InstrumentId, Symbol, Venue

def test_pickling_symbol():
    data = Symbol("Hello world")
    
    pickled = pickle.dumps(data)
    unpickled = pickle.loads(pickled)

    assert data == unpickled

def test_pickling_symbol():
    data = Venue("Hello world")
    
    pickled = pickle.dumps(data)
    unpickled = pickle.loads(pickled)

    assert data == unpickled

def test_pickling_instrument():
    data = InstrumentId.from_string("Hello.World")
    
    pickled = pickle.dumps(data)
    unpickled = pickle.loads(pickled)

    assert data == unpickled

def test_pickling_quote():
    venue = Venue("Something")
    instrument = InstrumentId(
        symbol = Symbol("hello world"),
        venue = venue
    )
    data = QuoteTick(instrument)
    
    pickled = pickle.dumps(data)
    unpickled = pickle.loads(pickled)

    assert data == unpickled
    assert data.instrument_id == unpickled.instrument_id
