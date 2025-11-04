use numpy::ndarray::Array3;
use numpy::{IntoPyArray, PyArray3};
use pdfium_render::prelude::*;
use pyo3::prelude::*;
use std::path::Path;

#[cfg(target_os = "macos")]
const PDFIUM_LIB_PATH: &str = "/usr/local/lib/libpdfium.dylib";
#[cfg(target_os = "linux")]
const PDFIUM_LIB_PATH: &str = "/usr/local/lib/libpdfium.so";

fn get_pdfium() -> PyResult<Pdfium> {
    let bindings = Pdfium::bind_to_library(Path::new(PDFIUM_LIB_PATH))
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    Ok(Pdfium::new(bindings))
}

fn load_pdf<'a>(pdfium: &'a Pdfium, py_pdf_bytes: &'a [u8]) -> PyResult<PdfDocument<'a>> {
    pdfium
        .load_pdf_from_byte_slice(py_pdf_bytes, None)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

fn render_page(
    page: &PdfPage,
    dpi: i32,
) -> Result<(Vec<u16>, usize, usize), Box<dyn std::error::Error>> {
    let render = page.render_with_config(
        &PdfRenderConfig::new()
            .set_target_width(dpi)
            .render_form_data(true),
    )?;

    let image = render.as_image();
    let width = image.width() as usize;
    let height = image.height() as usize;

    if width == 1 && height == 1 {
        return Err("Invalid image size 1x1".into());
    }

    let rgb = image.to_rgb8();
    let bgr_bytes = rgb_to_bgr(&rgb.into_raw());

    let bgr_u16: Vec<u16> = bgr_bytes.iter().map(|&x| (x as u16) * 257).collect();

    Ok((bgr_u16, width, height))
}

fn rgb_to_bgr(rgb: &[u8]) -> Vec<u8> {
    let mut bgr = rgb.to_vec();
    for chunk in bgr.chunks_exact_mut(3) {
        chunk.swap(0, 2);
    }
    bgr
}

fn page_to_pyarray(py: Python, page: &PdfPage) -> PyResult<Py<PyArray3<u16>>> {
    let (bgr_u16, width, height) = render_page(page, 1200)
        .or_else(|_| render_page(page, 100))
        .map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Failed rendering page: {}", e))
        })?;

    let array3d = Array3::from_shape_vec((height, width, 3), bgr_u16).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to create ndarray: {}", e))
    })?;

    Ok(array3d.into_pyarray_bound(py).into())
}

#[pyfunction]
fn pdf2img(py: Python, py_pdf_bytes: &[u8]) -> PyResult<Py<PyArray3<u16>>> {
    let pdfium = get_pdfium()?;
    let doc = load_pdf(&pdfium, py_pdf_bytes)?;
    let page = doc
        .pages()
        .get(0)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    page_to_pyarray(py, &page)
}

#[pyfunction]
fn pdf2imgs(py: Python, py_pdf_bytes: &[u8]) -> PyResult<Vec<Py<PyArray3<u16>>>> {
    let pdfium = get_pdfium()?;
    let doc = load_pdf(&pdfium, py_pdf_bytes)?;

    let mut results = Vec::new();
    for i in 0..doc.pages().len() {
        let page = doc.pages().get(i).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Page {} error: {}", i, e))
        })?;
        results.push(page_to_pyarray(py, &page)?);
    }

    Ok(results)
}

#[pymodule]
fn preprocess<'py>(_py: Python<'py>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pdf2img, m)?)?;
    m.add_function(wrap_pyfunction!(pdf2imgs, m)?)?;
    Ok(())
}
