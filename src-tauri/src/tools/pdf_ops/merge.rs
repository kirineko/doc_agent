use crate::tools::ToolError;
use lopdf::{Document, Object, ObjectId};
use std::collections::BTreeMap;
use std::path::Path;

pub fn merge_pdfs(inputs: &[&Path], out: &Path) -> Result<u32, ToolError> {
    if inputs.is_empty() {
        return Err(ToolError::InvalidArgs("至少需要一个输入 PDF".into()));
    }

    let mut documents = Vec::with_capacity(inputs.len());
    for input in inputs {
        let doc = Document::load(input)
            .map_err(|e| ToolError::Execution(format!("load {}: {e}", input.display())))?;
        documents.push(doc);
    }

    let mut max_id = 1;
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::with_version("1.5");

    for mut doc in documents {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        for (_, object_id) in doc.get_pages() {
            let obj = doc
                .get_object(object_id)
                .map_err(|e| ToolError::Execution(e.to_string()))?
                .to_owned();
            documents_pages.insert(object_id, obj);
        }
        documents_objects.extend(doc.objects);
    }

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in documents_objects.iter() {
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                catalog_object = Some((
                    catalog_object.map(|(id, _)| id).unwrap_or(*object_id),
                    object.clone(),
                ));
            }
            b"Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref obj)) = pages_object {
                        if let Ok(old_dict) = obj.as_dict() {
                            dictionary.extend(old_dict);
                        }
                    }
                    pages_object = Some((
                        pages_object.map(|(id, _)| id).unwrap_or(*object_id),
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                document.objects.insert(*object_id, object.clone());
            }
        }
    }

    let Some((catalog_id, catalog_obj)) = catalog_object else {
        return Err(ToolError::Execution("合并失败：未找到 Catalog".into()));
    };
    let Some((pages_id, pages_obj)) = pages_object else {
        return Err(ToolError::Execution("合并失败：未找到 Pages".into()));
    };

    if let Ok(dictionary) = pages_obj.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", documents_pages.len() as u32);
        dictionary.set(
            "Kids",
            documents_pages
                .keys()
                .map(|&id| Object::Reference(id))
                .collect::<Vec<_>>(),
        );
        document
            .objects
            .insert(pages_id, Object::Dictionary(dictionary));
    }

    for (object_id, object) in documents_pages {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_id);
            document
                .objects
                .insert(object_id, Object::Dictionary(dictionary));
        }
    }

    if let Ok(dictionary) = catalog_obj.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_id);
        dictionary.remove(b"Outlines");
        document
            .objects
            .insert(catalog_id, Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_id);
    document.max_id = document.objects.len() as u32;
    document.renumber_objects();
    document.adjust_zero_pages();
    document.compress();
    document
        .save(out)
        .map_err(|e| ToolError::Execution(format!("save {}: {e}", out.display())))?;

    let merged = Document::load(out)
        .map_err(|e| ToolError::Execution(format!("verify {}: {e}", out.display())))?;
    Ok(merged.get_pages().len() as u32)
}
