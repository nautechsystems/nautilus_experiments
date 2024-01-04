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
from experiments.data.objects import CyLogger
from core import set_global_log_collector

def test_cython_logging():
    CyLogger.info("cy: Yeehaw!")
    CyLogger.info("cy: Huffaw")
    CyLogger.info("cy: Green men")


def test_logging():
    logger = TempLogger("cowboy")
    logger.info("py: Yeehaw!")
    logger.info("py: Huffaw")
    logger.info("py: Green men")

# try with various RUST_LOG settings
# python test_logging
# RUST_LOG="core=debug" python test_logging
# RUST_LOG="core=info" python test_logging
if __name__ == "__main__":
    CyLogger.init()
    # set_global_log_collector()
    test_cython_logging()
    test_logging()
