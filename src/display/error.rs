use pyo3::create_exception;
use pyo3::exceptions::PyException;

create_exception!(display, DisplayError, PyException);
create_exception!(display, MonitorNotFoundError, DisplayError);
create_exception!(display, ResolutionNotSupportedError, DisplayError);
create_exception!(display, DisplayPermissionError, DisplayError);