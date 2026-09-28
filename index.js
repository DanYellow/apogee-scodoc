import fs from "node:fs";

import Papa from 'papaparse';


const STUDENTS_REGEX  = /(\d{8})[\w\t\s\n\/-]+/g;
const STUDENT_NIP_CODE = /\d{8}/g

const csvmaker = function (data) {
    // Empty array for storing the values
    csvRows = [];
    const headers = Object.keys(data);
    csvRows.push(headers.join(','));

    const values = Object.values(data).join(',');
    csvRows.push(values)

    return csvRows.join('\n')
}

const headerCSV = [
    "code_nip",
    "Nom",
    "Prénom",
    "UE2.1",
    "UE2.2",
    "UE2.3",
    "UE2.4",
    "UE2.5",
].join(',')

const loadApogeeTxtFile = async () => {
    const scodocGradesFileContent = await fs.readFileSync("7W29B2-4 MMI CREA FI S4.tmp.TXT");

    const strContent = scodocGradesFileContent.toString();
    const listStudents = strContent.match(STUDENTS_REGEX);
    const listStudentsNipCode = listStudents[0].match(STUDENT_NIP_CODE);

    return listStudentsNipCode;

    // fs.readFile("7W29B2-4 MMI CREA FI S4.TXT", function(err, content) {
    //     if (err) throw err;
    //     // Do Required Operations with the content which is the file data
        
    //     const strContent = content.toString();
    //     const listStudents = strContent.match(STUDENTS_REGEX);
    //     const listStudentsNipCode = listStudents[0].match(STUDENT_NIP_CODE)
    //     console.log("content", listStudentsNipCode)
    // });
}

;(async() => {
    const listStudentsNipCode = await loadApogeeTxtFile();
    
    const scodocGradesFile = await fs.readFileSync("./recap-BUT_Metiers_du_multimedia_et_de_linternet_semestre_1-2026-09-28.tmp.csv");

    const columnsToKeep = ['code_nip', 'Nom', 'Prénom', ]
    const REGEX_UE = /^UE\d/;

    Papa.parse(scodocGradesFile.toString(), {
        skipFirstNLines: 1,
        header: true,
        complete: ({ data }) => {
            const filtered = data.map(row =>
                Object.fromEntries(
                    Object.entries(row).filter(([key]) =>
                    {
                        return columnsToKeep.includes(key) ||
                        REGEX_UE.test(key)
                    })
                )
            );
            console.log(filtered[0])
            console.log(filtered[1])
            const listStudentsFiltered = filtered
                .filter((item) => listStudentsNipCode.includes(item.code_nip))
                .sort((a, b) => a["Nom"].localeCompare(b["Nom"]));

            const result = listStudentsFiltered.map(obj => {
                const newObj = {};

                for (const [key, value] of Object.entries(obj)) {
                    newObj[key] = value;
                    if (REGEX_UE.test(key)) {
                        newObj[`${key}_barème`] = "20";
                        newObj[`${key}_pts_jury`] = "";
                        newObj[`${key}_résultat`] = "";
                    }
                }

                return newObj;
            });

            const csv = Papa.unparse(result, {
                delimiter: ";",
                newline: "\r\n"
            });

            fs.writeFileSync(
                "./test.tmp.csv",
                "\uFEFF" + csv,
                "utf8"
            );
            // console.log(csv);
    
            // fs.writeFileSync("./test.tmp.csv", csv)
        },
    });
    // console.log("fff", scodocGrades)

    
})()

