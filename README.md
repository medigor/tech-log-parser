# tech-log-parser
Парсер технологического журнала 1С:Предприятие 8.

## Недостатки существующих способов
- Рекомендуемый способ 1С - это скрипты perl и регулярные выражения, это неудобно и медленно.
- В 8.3.25 появилась возможность настроить вывод в формате Json, но он имеет существенный недостаток: получаемый json может иметь дубли, что не позволяет десериализовать его простым способом.

## Состав проекта
Репозиторий содержит в себе несколько проектов:
- [Библиотека](parser) для парсинга технологического журнала, позже будет публиковаться крейт.
- [simple-bench](tests/simple-bench) - консольная утилита для тестирования скорости парсинга, парсит события, подсчитывает количество событий и выводит длительность.
- [Конвертер](converter) - консольная утилита для конвертации отдельных файлов или целых каталогов в формат json.
- [Внешняя компонента](addin) Native Api для использования из 1С:Предприятие.

## Конвертер
Конвертирует файлы и каталоги технологического журнала в формат json.  
Пример конвертации одного файла:
```sh
converter /path/to/23122609.log /path/to/23122609.json
```
Пример конвертации каталога:
```sh
converter /path/to/tech-logs /path/to/tech-logs-json
```
Формат (Пример одного события):
```json
{
    "Date": "2023-12-26T09:15:04.544001",
    "Duration": 1,
    "Name": "SCALL",
    "Level": 0,
    "Props": [["process", "rphost"],["OSThread","1"],["ClientID", "1"]]
}
```
Пример анализа тех. журнала в формате json.  
Если читать все события целиком, то это потребует много оперативной памяти - примерно в 10 раз больше, чем размер самого файла. Поэтому предлагается следующая схема чтения по одному событию:
```bsl
ЧтениеJSON = Новый ЧтениеJSON();
ЧтениеJSON.ОткрытьФайл(ИмяФайла);
ЧтениеJSON.Прочитать();

Если ЧтениеJSON.ТипТекущегоЗначения <> ТипЗначенияJSON.НачалоМассива Тогда
    ВызватьИсключение "Некорректный файл";
КонецЕсли;

Пока Истина Цикл
    Событие = ПрочитатьJSON(ЧтениеJSON);
    ЧтениеJSON.ТипТекущегоЗначения = ТипЗначенияJSON.КонецМассива Тогда
        Прервать;
    КонецЕсли;

    Дата = XMLЗначение(Тип("Дата"), Событие.Date);
    Имя = Событие.Name;
    Длительность = Событие.Duration; // В миллисекундах
    Каждого КлючИЗначение Из Событие.Props Цикл
        Ключ = КлючИЗначение[0];
        Значение = КлючИЗначение[1];
    КонецЦикла;
КонецЦикла;
ЧтениеJSON.Закрыть();
```

## Внешняя компонента
Содержит единственный метод:
```
ParseFile(ИмяФайла: Строка, Фильтр: ДвоичныеДанные, Количество: Число): ДвоичныеДанные
```
Параметры:
- `ИмяФайла` - Строка - путь к файлу технологического журнала, имя файла должно оставаться оригинальным, т.к. из него формируется дата события.
- `Фильтр` - ДвоичныеДанные - Массив из фильтров кодированный в `Json`. Событие добавляется только если оно удовлетворяет всем условиям. Фильтр формируется так, чтобы удовлетворять типу `Filter` из модуля [filters.rs](addin/src/filters.rs), также см. [пример](addin/conf1c/DataProcessors/ТехЖурнал/Forms/Форма/Ext/Form/Module.bsl). Для регулярных выражений по умолчанию настроен регистронезависимый поиск (можно отключить с помощью шаблона `(?-i:<regex>)`) и точка соответствует также символу перевода строки (можно отключить с помощью шаблона `(?-s:<regex>)`)
Пример фильтра:
    ```json
    [
        {"Duration":{"GreaterOrEqual":123}},
        {"Name":{"Equal":"EXCP"}},
        {"Prop":{"Name":"process","Filter":{"Equal":"rphost"}}},
        {"Prop":{"Name":"Sql","Filter":{"Match":"^select"}}}
    ```
- `Количество` - Число - количество событий, которые будут получены из файла, если 0 - то будут прочитаны все события.

Возвращаемое значение:
- ДвоичныеДанные - список событий в формате `Json`, формат как в конвертере.

Метод может бросать исключение, например если передан невалидный `Json`, текст ошибки можно получить из свойства `LastError`.  
Также проект содержит тестовую [конфигурацию](addin/conf1c), выгруженную из конфигуратора, платформа 8.3.23. Весь код расположен в [форме обработки](addin/conf1c/DataProcessors/ТехЖурнал/Forms/Форма/Ext/Form/Module.bsl).

## Сборка проекта
См. [инструкцию](https://github.com/medigor/rust-build-scripts).
