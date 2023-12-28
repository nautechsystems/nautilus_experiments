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

from core import TempLogger
from core import LogGuard
from core import set_global_log_collector

def test_logging():
    guard = set_global_log_collector("debug", None, None)
    logger = TempLogger("cowboy")
    logger.debug("Yeehaw!")
    guard.time.increment_time(40)
    logger.info("Huffaw")
    guard.time.increment_time(80)
    logger.warn("Bleh")
    guard.time.live()
    logger.error("Wololo")
    ignore_logger = TempLogger("alien")
    ignore_logger.debug("Green men")
    ignore_logger.warn("Pew pew")
    ignore_logger.error("Zoom zoom")

# try with various RUST_LOG settings
# python test_logging
# RUST_LOG="core=debug" python test_logging
# RUST_LOG="core=info" python test_logging
if __name__ == "__main__":
    test_logging()
